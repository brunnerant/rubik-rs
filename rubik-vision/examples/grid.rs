use std::f32::consts::TAU;
use std::ffi::OsStr;

use opencv::core::Vector;
use opencv::imgcodecs::{IMREAD_COLOR, imread, imwrite};
use plotters::backend::SVGBackend;
use plotters::chart::{ChartBuilder, LabelAreaPosition};
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::series::PointSeries;
use plotters::style::{BLUE, WHITE};
use rubik_vision::grid::lines;
use workspace_root::get_workspace_root;

fn main() {
    let img_folder = get_workspace_root().join("data/vision");
    let out_folder = img_folder.join("out");
    std::fs::create_dir_all(&out_folder).unwrap();

    for img in std::fs::read_dir(img_folder).unwrap() {
        let img = img.unwrap().path();
        if !img.is_file() || img.extension() != Some(OsStr::new("png")) {
            continue;
        }
        let out = out_folder.join(img.file_name().unwrap());
        let out_graph = out_folder.join(img.file_name().unwrap()).with_extension("svg");
        let mut img = imread(img.to_str().unwrap(), IMREAD_COLOR).unwrap();
        let lines = lines(&mut img).unwrap();
        imwrite(out.to_str().unwrap(), &img, &Vector::new()).unwrap();

        let root_area = SVGBackend::new(&out_graph, (600, 400)).into_drawing_area();
        root_area.fill(&WHITE).unwrap();
        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption("Scatter Plot Demo", ("sans-serif", 40))
            .build_cartesian_2d(0.0..TAU, 0.0f32..500.0).unwrap();
        ctx.configure_mesh().draw().unwrap();
        ctx.draw_series(
            PointSeries::<_, _, Circle<_, _>, _>::new(
                lines.iter().map(|l| (l.theta, l.rho)),
                5,
                &BLUE,
            )
        ).unwrap();
    }
}
