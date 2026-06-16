use std::f32::consts::{PI, TAU};
use std::ffi::OsStr;

use opencv::core::Vector;
use opencv::imgcodecs::{IMREAD_COLOR, imread, imwrite};
use plotters::backend::SVGBackend;
use plotters::chart::{ChartBuilder, LabelAreaPosition};
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::series::{LineSeries, PointSeries};
use plotters::style::{BLUE, CYAN, GREEN, MAGENTA, RED, WHITE};
use rubik_vision::grid::{find_peaks, find_two_peaks, kde, lines};
use workspace_root::get_workspace_root;

const BANDWIDTH: f32 = 0.1;
const KDE_RESOLUTION: usize = 512;
const RHO_MAX: f32 = 500.0;
const THETA_TOLERANCE: f32 = 10.0 * PI / 180.0;
const RHO_BANDWIDTH: f32 = 5.0;

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
        let out_svg = out_folder
            .join(path.file_name().unwrap())
            .with_extension("svg");

        let mut img = imread(path.to_str().unwrap(), IMREAD_COLOR).unwrap();
        let detected = lines(&mut img).unwrap();
        imwrite(out_img.to_str().unwrap(), &img, &Vector::new()).unwrap();

        let thetas: Vec<f32> = detected.iter().map(|l| l.theta).collect();
        let theta_peaks = find_two_peaks(&thetas, BANDWIDTH);

        let name = path.file_name().unwrap().to_string_lossy();

        // Filter lines into two orientation groups and detect rho peaks.
        let (rhos1, rhos2, rho_peaks1, rho_peaks2) = match theta_peaks {
            None => {
                println!("{name}: no grid detected");
                (vec![], vec![], vec![], vec![])
            }
            Some([t1, t2]) => {
                println!(
                    "{name}: grid at {:.1}° and {:.1}°",
                    t1.to_degrees(),
                    t2.to_degrees()
                );
                let r1: Vec<f32> = detected
                    .iter()
                    .filter(|l| (l.theta - t1).abs() < THETA_TOLERANCE)
                    .map(|l| l.rho)
                    .collect();
                let r2: Vec<f32> = detected
                    .iter()
                    .filter(|l| (l.theta - t2).abs() < THETA_TOLERANCE)
                    .map(|l| l.rho)
                    .collect();
                let p1 = find_peaks(&r1, RHO_BANDWIDTH, 0.5);
                let p2 = find_peaks(&r2, RHO_BANDWIDTH, 0.5);
                (r1, r2, p1, p2)
            }
        };

        // Rho KDE over [0, RHO_MAX] for display (aligns with scatter y-axis).
        let rho_kde1 = kde(&rhos1, 0.0, RHO_MAX, RHO_BANDWIDTH, KDE_RESOLUTION);
        let rho_kde2 = kde(&rhos2, 0.0, RHO_MAX, RHO_BANDWIDTH, KDE_RESOLUTION);
        let rho_kde1_max = rho_kde1.iter().cloned().fold(0.0f32, f32::max);
        let rho_kde2_max = rho_kde2.iter().cloned().fold(0.0f32, f32::max);

        // --- Layout: left panel | scatter | right panel ---
        let root = SVGBackend::new(&out_svg, (800, 400)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let panels = root.split_by_breakpoints([100u32, 700u32], [] as [u32; 0]);

        // Left panel: rho KDE for group 1, reversed x so density grows outward.
        {
            let x_max = rho_kde1_max.max(1.0);
            let mut panel = ChartBuilder::on(&panels[0])
                .set_label_area_size(LabelAreaPosition::Left, 30)
                .build_cartesian_2d(x_max..0.0_f32, 0.0_f32..RHO_MAX)
                .unwrap();
            panel
                .configure_mesh()
                .disable_x_mesh()
                .disable_y_mesh()
                .draw()
                .unwrap();
            panel
                .draw_series(LineSeries::new(
                    (0..KDE_RESOLUTION).map(|i| {
                        let rho = i as f32 * RHO_MAX / (KDE_RESOLUTION - 1) as f32;
                        (rho_kde1[i], rho)
                    }),
                    &CYAN,
                ))
                .unwrap();
        }

        // Center panel: scatter + theta KDE + theta peak markers + rho peak markers.
        {
            let mut ctx = ChartBuilder::on(&panels[1])
                .set_label_area_size(LabelAreaPosition::Bottom, 30)
                .caption(name.as_ref(), ("sans-serif", 20))
                .build_cartesian_2d(0.0f32..TAU, 0.0f32..RHO_MAX)
                .unwrap();
            ctx.configure_mesh().draw().unwrap();

            ctx.draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
                detected.iter().map(|l| (l.theta, l.rho)),
                4,
                &BLUE,
            ))
            .unwrap();

            // Theta KDE overlay (scaled to fit rho axis).
            if !thetas.is_empty() {
                let density = kde(&thetas, 0.0, TAU, BANDWIDTH, KDE_RESOLUTION);
                let peak_density = density.iter().cloned().fold(0.0f32, f32::max);
                let scale = if peak_density > 0.0 {
                    RHO_MAX * 0.8 / peak_density
                } else {
                    1.0
                };
                ctx.draw_series(LineSeries::new(
                    (0..KDE_RESOLUTION).map(|i| {
                        let theta = i as f32 * TAU / (KDE_RESOLUTION - 1) as f32;
                        (theta, density[i] * scale)
                    }),
                    &RED,
                ))
                .unwrap();
            }

            // Theta peak markers.
            if let Some(peaks) = theta_peaks {
                for peak in peaks {
                    ctx.draw_series(LineSeries::new([(peak, 0.0), (peak, RHO_MAX)], &GREEN))
                        .unwrap();
                }
            }

            // Rho peak markers (horizontal lines).
            for &rho in &rho_peaks1 {
                ctx.draw_series(LineSeries::new([(0.0, rho), (TAU, rho)], &CYAN))
                    .unwrap();
            }
            for &rho in &rho_peaks2 {
                ctx.draw_series(LineSeries::new([(0.0, rho), (TAU, rho)], &MAGENTA))
                    .unwrap();
            }
        }

        // Right panel: rho KDE for group 2, normal x so density grows outward.
        {
            let x_max = rho_kde2_max.max(1.0);
            let mut panel = ChartBuilder::on(&panels[2])
                .set_label_area_size(LabelAreaPosition::Right, 30)
                .build_cartesian_2d(0.0_f32..x_max, 0.0_f32..RHO_MAX)
                .unwrap();
            panel
                .configure_mesh()
                .disable_x_mesh()
                .disable_y_mesh()
                .draw()
                .unwrap();
            panel
                .draw_series(LineSeries::new(
                    (0..KDE_RESOLUTION).map(|i| {
                        let rho = i as f32 * RHO_MAX / (KDE_RESOLUTION - 1) as f32;
                        (rho_kde2[i], rho)
                    }),
                    &MAGENTA,
                ))
                .unwrap();
        }
    }
}
