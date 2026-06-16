use std::f32::consts::{PI, TAU};
use std::ffi::OsStr;

use opencv::core::{
    AlgorithmHint, BORDER_CONSTANT, BORDER_DEFAULT, Mat, MatTraitConst, Point, Point2f, Size,
    Vec4f, VecN, Vector, bitwise_and, no_array,
};
use opencv::imgcodecs::{IMREAD_COLOR, imread, imwrite};
use opencv::imgproc::{
    ADAPTIVE_THRESH_MEAN_C, CHAIN_APPROX_SIMPLE, COLOR_BGR2GRAY, INTER_LINEAR, LINE_AA,
    MORPH_OPEN, MORPH_RECT, RETR_LIST, THRESH_BINARY_INV, adaptive_threshold, canny, circle,
    cvt_color, find_contours, get_rotation_matrix_2d, get_structuring_element, hough_lines_p,
    moments, morphology_ex, warp_affine,
};
use rubik_vision::grid::find_two_peaks;
use workspace_root::get_workspace_root;

// Morphological kernel length = image dimension / this divisor.
const H_KERNEL_DIV: i32 = 9;
const V_KERNEL_DIV: i32 = 9;

// Adaptive threshold parameters.
const BLOCK_SIZE: i32 = 21;
const THRESH_C: f64 = 10.0;

// Clustering: centroids within ROW_TOL pixels in y belong to the same row.
const ROW_TOL: f32 = 15.0;

// KDE bandwidth for theta peak detection.
const THETA_BANDWIDTH: f32 = 0.1;

fn main() {
    let img_folder = get_workspace_root().join("data/vision");
    let out_folder = img_folder.join("out_morph");
    std::fs::create_dir_all(&out_folder).unwrap();

    for entry in std::fs::read_dir(&img_folder).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() || path.extension() != Some(OsStr::new("png")) {
            continue;
        }

        let name = path.file_name().unwrap().to_string_lossy();
        let out_img = out_folder.join(path.file_name().unwrap());

        let mut img = imread(path.to_str().unwrap(), IMREAD_COLOR).unwrap();

        match detect_grid(&img) {
            None => println!("{name}: no grid found"),
            Some((grid, angle_deg)) => {
                let cell_rows = grid.len().saturating_sub(1);
                let cell_cols = grid.first().map_or(0, |r| r.len().saturating_sub(1));
                let n_corners: usize = grid.iter().map(|r| r.len()).sum();
                println!(
                    "{name}: {cell_rows}×{cell_cols} cells ({n_corners} corners, rotated {angle_deg:.1}°)"
                );
                draw_grid(&mut img, &grid).unwrap();
            }
        }

        imwrite(out_img.to_str().unwrap(), &img, &Vector::new()).unwrap();
    }
}

/// Full pipeline. Returns the grid corners in original image coordinates and
/// the rotation angle that was applied internally.
fn detect_grid(img: &Mat) -> Option<(Vec<Vec<(f32, f32)>>, f32)> {
    let mut gray = Mat::default();
    cvt_color(img, &mut gray, COLOR_BGR2GRAY, 0, AlgorithmHint::ALGO_HINT_DEFAULT).unwrap();

    let cx = img.cols() as f32 / 2.0;
    let cy = img.rows() as f32 / 2.0;

    // 1. Estimate how much the grid is rotated away from axis-aligned.
    //    Falls back to 0° (no-op rotation) if angle detection fails.
    let angle_deg = -dominant_angle(&gray).unwrap_or(0.0);

    // 2. Rotate the gray image so that grid lines become horizontal / vertical.
    let rotated = rotate_mat(&gray, angle_deg, cx, cy);

    // 3. Adaptive threshold: grid lines (dark borders) → white on black.
    let mut thresh = Mat::default();
    adaptive_threshold(
        &rotated,
        &mut thresh,
        255.0,
        ADAPTIVE_THRESH_MEAN_C,
        THRESH_BINARY_INV,
        BLOCK_SIZE,
        THRESH_C,
    )
    .unwrap();

    // 4. Horizontal line mask: MORPH_OPEN with a wide 1-pixel-tall kernel
    //    discards anything shorter than h_len, keeping only true grid edges.
    let h_len = (img.cols() / H_KERNEL_DIV).max(10);
    let h_kernel =
        get_structuring_element(MORPH_RECT, Size::new(h_len, 1), Point::new(-1, -1)).unwrap();
    let mut h_lines = Mat::default();
    morphology_ex(
        &thresh,
        &mut h_lines,
        MORPH_OPEN,
        &h_kernel,
        Point::new(-1, -1),
        1,
        BORDER_DEFAULT,
        VecN::new(0.0, 0.0, 0.0, 0.0),
    )
    .unwrap();

    // 5. Vertical line mask.
    let v_len = (img.rows() / V_KERNEL_DIV).max(10);
    let v_kernel =
        get_structuring_element(MORPH_RECT, Size::new(1, v_len), Point::new(-1, -1)).unwrap();
    let mut v_lines = Mat::default();
    morphology_ex(
        &thresh,
        &mut v_lines,
        MORPH_OPEN,
        &v_kernel,
        Point::new(-1, -1),
        1,
        BORDER_DEFAULT,
        VecN::new(0.0, 0.0, 0.0, 0.0),
    )
    .unwrap();

    // 6. Intersection mask: pixels present in both line masks.
    let mut mask = Mat::default();
    bitwise_and(&h_lines, &v_lines, &mut mask, &no_array()).unwrap();

    // 7. Centroid of each intersection blob.
    let mut contours: Vector<Vector<Point>> = Vector::new();
    find_contours(
        &mut mask,
        &mut contours,
        RETR_LIST,
        CHAIN_APPROX_SIMPLE,
        Point::default(),
    )
    .unwrap();

    let pts: Vec<(f32, f32)> = contours
        .iter()
        .filter_map(|c| {
            let m = moments(&c, false).ok()?;
            if m.m00 == 0.0 {
                return None;
            }
            Some(((m.m10 / m.m00) as f32, (m.m01 / m.m00) as f32))
        })
        .collect();

    if pts.len() < 4 {
        return None;
    }

    // 8. Cluster into rows (still in the rotated image's coordinate system).
    let grid_rot = cluster_into_grid(pts, ROW_TOL);

    // 9. Rotate each corner back into the original image coordinate system.
    let grid = grid_rot
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|(x, y)| rotate_pt_back(x, y, cx, cy, angle_deg))
                .collect()
        })
        .collect();

    Some((grid, angle_deg))
}

/// Detects the dominant rotation needed to make the grid axis-aligned.
///
/// Uses Canny + HoughLinesP to find line segments, converts them to Hough
/// normal angles θ, finds the two orientation peaks via KDE, then picks
/// the peak whose line family is closest to horizontal and returns the
/// angle (in degrees) needed to rotate those lines flat.
fn dominant_angle(gray: &Mat) -> Option<f32> {
    let mut edges = Mat::default();
    canny(gray, &mut edges, 100.0, 200.0, 3, false).unwrap();

    let mut segs: Vector<Vec4f> = Vector::new();
    hough_lines_p(&edges, &mut segs, 1.0, 1.0f64.to_radians(), 50, 20.0, 5.0).unwrap();

    if segs.is_empty() {
        return None;
    }

    // Same θ convention as the library's Line::from_segment.
    let thetas: Vec<f32> = segs
        .iter()
        .map(|l| {
            let (dx, dy) = (l[2] - l[0], l[3] - l[1]);
            let raw = (dx.atan2(-dy) + TAU) % TAU;
            (raw + 0.5 * PI) % TAU
        })
        .collect();

    let [t1, t2] = find_two_peaks(&thetas, THETA_BANDWIDTH)?;

    // Pick the orientation family closest to horizontal (θ = π/2).
    let t = if (t1 - PI / 2.0).abs() <= (t2 - PI / 2.0).abs() {
        t1
    } else {
        t2
    };

    // Positive result → rotate CCW in OpenCV convention → straightens CW-tilted lines.
    Some((PI / 2.0 - t).to_degrees())
}

/// Rotates `mat` by `angle_deg` around `(cx, cy)`, keeping the same canvas size.
/// Black pixels fill any newly exposed border regions.
fn rotate_mat(mat: &Mat, angle_deg: f32, cx: f32, cy: f32) -> Mat {
    let m = get_rotation_matrix_2d(Point2f::new(cx, cy), angle_deg as f64, 1.0).unwrap();
    let mut out = Mat::default();
    warp_affine(
        mat,
        &mut out,
        &m,
        Size::new(mat.cols(), mat.rows()),
        INTER_LINEAR,
        BORDER_CONSTANT,
        VecN::new(0.0, 0.0, 0.0, 0.0),
    )
    .unwrap();
    out
}

/// Maps a point from the rotated image back to the original image coordinate system.
///
/// `getRotationMatrix2D(center, θ, 1)` applies the matrix [[cosθ, sinθ], [−sinθ, cosθ]].
/// Its inverse (transpose for orthogonal matrices) is [[cosθ, −sinθ], [sinθ, cosθ]].
fn rotate_pt_back(x: f32, y: f32, cx: f32, cy: f32, angle_deg: f32) -> (f32, f32) {
    let theta = angle_deg.to_radians();
    let c = theta.cos();
    let s = theta.sin();
    let dx = x - cx;
    let dy = y - cy;
    (c * dx - s * dy + cx, s * dx + c * dy + cy)
}

/// Sorts a flat list of centroids into a row-major 2-D grid.
fn cluster_into_grid(mut pts: Vec<(f32, f32)>, row_tol: f32) -> Vec<Vec<(f32, f32)>> {
    pts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut rows: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut row = vec![pts[0]];

    for &pt in &pts[1..] {
        let mean_y = row.iter().map(|p| p.1).sum::<f32>() / row.len() as f32;
        if (pt.1 - mean_y).abs() < row_tol {
            row.push(pt);
        } else {
            row.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            rows.push(row);
            row = vec![pt];
        }
    }
    row.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    rows.push(row);
    rows
}

/// Draws green circles at corners and blue dots at cell centres.
fn draw_grid(img: &mut Mat, grid: &[Vec<(f32, f32)>]) -> opencv::Result<()> {
    for row in grid {
        for &(x, y) in row {
            circle(
                img,
                Point::new(x as i32, y as i32),
                5,
                VecN::new(0.0, 255.0, 0.0, 1.0),
                2,
                LINE_AA,
                0,
            )?;
        }
    }

    let n_rows = grid.len().saturating_sub(1);
    for i in 0..n_rows {
        let n_cols = grid[i].len().saturating_sub(1).min(grid[i + 1].len().saturating_sub(1));
        for j in 0..n_cols {
            let cx = (grid[i][j].0
                + grid[i][j + 1].0
                + grid[i + 1][j].0
                + grid[i + 1][j + 1].0)
                / 4.0;
            let cy = (grid[i][j].1
                + grid[i][j + 1].1
                + grid[i + 1][j].1
                + grid[i + 1][j + 1].1)
                / 4.0;
            circle(
                img,
                Point::new(cx as i32, cy as i32),
                4,
                VecN::new(255.0, 0.0, 0.0, 1.0),
                -1,
                LINE_AA,
                0,
            )?;
        }
    }

    Ok(())
}
