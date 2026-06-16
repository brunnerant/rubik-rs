use std::f32::consts::{PI, TAU};

use opencv::{
    core::{BORDER_DEFAULT, CV_FP16, Mat, MatTraitConst, Point, Scalar, Vec4f, VecN, Vector},
    imgproc::{LINE_AA, canny, dilate, erode, hough_lines_p, line},
};

#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub rho: f32,
    pub theta: f32,
}

impl Line {
    pub fn from_segment(a: (f32, f32), b: (f32, f32)) -> Self {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let mut theta = (dx.atan2(-dy) + TAU) % TAU;
        let mut rho = a.0 * theta.cos() + a.1 * theta.sin();
        if rho < 0.0 {
            rho = -rho;
            theta = (theta + PI) % TAU;
        }
        Self { rho, theta: (theta + 0.5 * PI) % TAU }
    }
}

pub fn lines(img: &mut Mat) -> opencv::Result<Vec<Line>> {
    let mut edges =
        Mat::new_rows_cols_with_default(img.rows(), img.cols(), img.typ(), Scalar::default())?;
    let mut edges_dil: Mat =
        Mat::new_rows_cols_with_default(img.rows(), img.cols(), img.typ(), Scalar::default())?;
        canny(img, &mut edges, 150.0, 250.0, 3, false).unwrap();
    let kernel = Mat::new_rows_cols_with_default(3,3, CV_FP16, 1.0.into()).unwrap();
    dilate(
        &mut edges,
        &mut edges_dil,
        &kernel,
        Point::new(-1, -1),
        2,
        BORDER_DEFAULT,
        VecN::new(0.0, 0.0, 0.0, 0.0),
    )?;
    erode(
        &mut edges_dil,
        &mut edges,
        &kernel,
        Point::new(-1, -1),
        1,
        BORDER_DEFAULT,
        VecN::new(0.0, 0.0, 0.0, 0.0),
    )
    .unwrap();
    let mut lines = Vector::<Vec4f>::new();
    hough_lines_p(
        &edges,
        &mut lines,
        1.0,
        1.0f64.to_radians(),
        100,
        30.0,
        10.0,
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
