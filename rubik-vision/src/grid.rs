use std::f32::consts::{PI, TAU};

use opencv::{
    core::{BORDER_DEFAULT, CV_FP16, Mat, MatTraitConst, Point, Scalar, Vec4f, VecN, Vector},
    imgproc::{LINE_AA, canny, dilate, erode, hough_lines_p, line},
};

/// Evaluates a Gaussian KDE at `resolution` evenly-spaced points from `min` to `max`.
pub fn kde(data: &[f32], min: f32, max: f32, bandwidth: f32, resolution: usize) -> Vec<f32> {
    let step = (max - min) / (resolution - 1) as f32;
    let h2 = 2.0 * bandwidth * bandwidth;
    (0..resolution)
        .map(|i| {
            let x = min + i as f32 * step;
            data.iter().map(|&xi| (-(x - xi).powi(2) / h2).exp()).sum()
        })
        .collect()
}

/// Returns the minimum density encountered walking left from `peak` until a higher value is
/// found (or the edge is reached).
fn col_to_left(density: &[f32], peak: usize) -> f32 {
    let mut col = f32::INFINITY;
    for i in (0..peak).rev() {
        if density[i] > density[peak] {
            break;
        }
        col = col.min(density[i]);
    }
    col
}

/// Returns the minimum density encountered walking right from `peak` until a higher value is
/// found (or the edge is reached).
fn col_to_right(density: &[f32], peak: usize) -> f32 {
    let mut col = f32::INFINITY;
    for i in (peak + 1)..density.len() {
        if density[i] > density[peak] {
            break;
        }
        col = col.min(density[i]);
    }
    col
}

/// Returns all prominent peaks in a 1D dataset, sorted by position (ascending).
///
/// Prominence is measured topographically: for each local maximum, walk outward in both
/// directions until reaching a higher peak or the edge of the data, and record the minimum
/// density seen on each side. The key col is the higher of the two side minima (the easier
/// path to any higher terrain). Prominence = peak density − key col.
///
/// Only peaks whose prominence exceeds `min_prominence × (density_max − density_min)` are
/// returned. `min_prominence` is in [0, 1]: 0 keeps every local maximum, 1 keeps only peaks
/// that span the entire density range above their surroundings.
pub fn find_peaks(data: &[f32], bandwidth: f32, min_prominence: f32) -> Vec<f32> {
    const RESOLUTION: usize = 512;

    if data.len() < 2 {
        return vec![];
    }

    let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    if min >= max {
        return vec![];
    }

    // Extend the evaluation window so the density decays naturally before the edges.
    // At 3 bandwidths from the nearest data point a Gaussian contributes < 1%.
    let lo = min - 3.0 * bandwidth;
    let hi = max + 3.0 * bandwidth;
    let step = (hi - lo) / (RESOLUTION - 1) as f32;
    let density = kde(data, lo, hi, bandwidth, RESOLUTION);
    let d_min = density.iter().cloned().fold(f32::INFINITY, f32::min);
    let d_max = density.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let threshold = min_prominence * (d_max - d_min);

    (1..RESOLUTION - 1)
        .filter(|&i| density[i] > density[i - 1] && density[i] > density[i + 1])
        .filter(|&i| {
            let key_col = col_to_left(&density, i).max(col_to_right(&density, i));
            density[i] - key_col > threshold
        })
        .map(|i| lo + i as f32 * step)
        .collect()
}

/// Returns the two prominent peaks, or `None` if the data doesn't contain exactly two.
pub fn find_two_peaks(data: &[f32], bandwidth: f32) -> Option<[f32; 2]> {
    match find_peaks(data, bandwidth, 0.3)[..] {
        [a, b] => Some([a, b]),
        _ => None,
    }
}

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
        Self {
            rho,
            theta: (theta + 0.5 * PI) % TAU,
        }
    }
}

pub fn lines(img: &mut Mat) -> opencv::Result<Vec<Line>> {
    let mut edges =
        Mat::new_rows_cols_with_default(img.rows(), img.cols(), img.typ(), Scalar::default())?;
    let mut edges_dil: Mat =
        Mat::new_rows_cols_with_default(img.rows(), img.cols(), img.typ(), Scalar::default())?;
    canny(img, &mut edges, 150.0, 250.0, 3, false).unwrap();
    let kernel = Mat::new_rows_cols_with_default(3, 3, CV_FP16, 1.0.into()).unwrap();
    dilate(
        &edges,
        &mut edges_dil,
        &kernel,
        Point::new(-1, -1),
        2,
        BORDER_DEFAULT,
        VecN::new(0.0, 0.0, 0.0, 0.0),
    )?;
    erode(
        &edges_dil,
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
            LINE_AA,
            0,
        )?;
    }
    Ok(out_lines)
}
