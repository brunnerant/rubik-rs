use std::f32::consts::{PI, TAU};

use opencv::{
    core::{BORDER_DEFAULT, CV_FP16, Mat, MatTraitConst, Point, Scalar, Vec4f, VecN, Vector},
    imgproc::{LINE_AA, canny, dilate, hough_lines_p, line},
};

#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub theta: f32,
    pub rho: f32,
}

impl Line {
    pub fn from_segment(a: (f32, f32), b: (f32, f32)) -> Self {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let mut theta = (dx.atan2(dy) + TAU) % TAU;
        let mut rho = a.1 * theta.cos() - a.0 * theta.sin();
        if rho < 0.0 {
            rho = -rho;
            theta = (theta + PI) % TAU;
        }
        Self { theta, rho }
    }
}

pub fn lines(img: &mut Mat) -> opencv::Result<Vec<Line>> {
    let mut edges =
        Mat::new_rows_cols_with_default(img.rows(), img.cols(), img.typ(), Scalar::default())?;
    let mut edges_dil: Mat =
        Mat::new_rows_cols_with_default(img.rows(), img.cols(), img.typ(), Scalar::default())?;
    let kernel = Mat::new_rows_cols_with_default(3, 3, CV_FP16, 1.0.into()).unwrap();
    canny(img, &mut edges, 100.0, 200.0, 3, false).unwrap();
    dilate(
        &mut edges,
        &mut edges_dil,
        &kernel,
        Point::new(-1, -1),
        1,
        BORDER_DEFAULT,
        VecN::new(0.0, 0.0, 0.0, 0.0),
    )?;
    let mut lines = Vector::<Vec4f>::new();
    hough_lines_p(
        &edges_dil,
        &mut lines,
        1.0,
        1.0f64.to_radians(),
        50,
        100.0,
        25.0,
    )?;
    let mut out_lines = Vec::with_capacity(lines.len());
    for l in lines {
        out_lines.push(Line::from_segment((l[0], l[1]), (l[2], l[3])));
        line(
            img,
            Point::new(l[0] as i32, l[1] as i32),
            Point::new(l[2] as i32, l[3] as i32),
            VecN::new(0.0, 0.0, 255.0, 1.0),
            1,
            LINE_AA.into(),
            0,
        )?;
    }
    Ok(out_lines)
}
