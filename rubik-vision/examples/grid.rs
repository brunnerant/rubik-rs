use std::f32::consts::TAU;
use std::ffi::OsStr;

use opencv::core::Vector;
use opencv::imgcodecs::{IMREAD_COLOR, imread, imwrite};
use plotters::backend::SVGBackend;
use plotters::chart::{ChartBuilder, LabelAreaPosition};
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::series::{LineSeries, PointSeries};
use plotters::style::{BLUE, GREEN, RED, WHITE};
use rubik_vision::grid::{find_two_peaks, kde, lines};
use workspace_root::get_workspace_root;

const BANDWIDTH: f32 = 0.1;
const KDE_RESOLUTION: usize = 512;
const RHO_MAX: f32 = 500.0;

fn main() {
    let img_folder = get_workspace_root().join("data/vision");
    let out_folder = img_folder.join("out");
    std::fs::create_dir_all(&out_folder).unwrap();

    for entry in std::fs::read_dir(&img_folder).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() || path.extension() != Some(OsStr::new("png")) {
            continue;
        }

        let out_img = out_folder.join(path.file_name().unwrap());
        let out_svg = out_folder.join(path.file_name().unwrap()).with_extension("svg");

        let mut img = imread(path.to_str().unwrap(), IMREAD_COLOR).unwrap();
        let detected = lines(&mut img).unwrap();
        imwrite(out_img.to_str().unwrap(), &img, &Vector::new()).unwrap();

        let thetas: Vec<f32> = detected.iter().map(|l| l.theta).collect();
        let peaks = find_two_peaks(&thetas, BANDWIDTH);

        let name = path.file_name().unwrap().to_string_lossy();
        match peaks {
            Some([a, b]) => println!("{name}: grid at {:.1}° and {:.1}°", a.to_degrees(), b.to_degrees()),
            None => println!("{name}: no grid detected"),
        }

        // --- Chart ---
        let root = SVGBackend::new(&out_svg, (600, 400)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut ctx = ChartBuilder::on(&root)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption(name.as_ref(), ("sans-serif", 28))
            .build_cartesian_2d(0.0f32..TAU, 0.0f32..RHO_MAX)
            .unwrap();
        ctx.configure_mesh().draw().unwrap();

        // Raw (theta, rho) scatter
        ctx.draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
            detected.iter().map(|l| (l.theta, l.rho)),
            4,
            &BLUE,
        ))
        .unwrap();

        // KDE of theta values (scaled to fit the rho axis)
        if !thetas.is_empty() {
            let density = kde(&thetas, 0.0, TAU, BANDWIDTH, KDE_RESOLUTION);
            let peak_density = density.iter().cloned().fold(0.0f32, f32::max);
            let scale = if peak_density > 0.0 { RHO_MAX * 0.8 / peak_density } else { 1.0 };
            ctx.draw_series(LineSeries::new(
                (0..KDE_RESOLUTION).map(|i| {
                    let theta = i as f32 * TAU / (KDE_RESOLUTION - 1) as f32;
                    (theta, density[i] * scale)
                }),
                &RED,
            ))
            .unwrap();
        }

        // Vertical markers at detected peak positions
        if let Some(peaks) = peaks {
            for peak in peaks {
                ctx.draw_series(LineSeries::new(
                    [(peak, 0.0), (peak, RHO_MAX)],
                    &GREEN,
                ))
                .unwrap();
            }
        }
    }
}
