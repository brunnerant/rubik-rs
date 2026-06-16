use std::f32::consts::{PI, TAU};
use std::ffi::OsStr;

use opencv::core::{Point, VecN, Vector};
use opencv::imgcodecs::{IMREAD_COLOR, imread, imwrite};
use opencv::imgproc::{LINE_AA, circle};
use rubik_vision::grid::{find_peaks, find_two_peaks, lines};
use workspace_root::get_workspace_root;

const THETA_BANDWIDTH: f32 = 0.1;
const THETA_TOLERANCE: f32 = 10.0_f32.to_radians();
const RHO_BANDWIDTH: f32 = 5.0;

// ---------------------------------------------------------------------------
// Grid types
// ---------------------------------------------------------------------------

struct Grid {
    theta1: f32,
    theta2: f32,
    /// Sorted rho values for the theta1 family of lines.
    _rhos1: Vec<f32>,
    /// Sorted rho values for the theta2 family of lines.
    _rhos2: Vec<f32>,
    /// `intersections[i][j]` = pixel (x, y) where rhos1[i] meets rhos2[j].
    intersections: Vec<Vec<(f32, f32)>>,
}

impl Grid {
    fn rows(&self) -> usize {
        self.intersections.len()
    }

    fn cols(&self) -> usize {
        self.intersections.first().map_or(0, |r| r.len())
    }

    /// Center of each cell bounded by intersection rows i..=i+1, cols j..=j+1.
    fn cell_centers(&self) -> Vec<Vec<(f32, f32)>> {
        let rows = self.rows().saturating_sub(1);
        let cols = self.cols().saturating_sub(1);
        (0..rows)
            .map(|i| {
                (0..cols)
                    .map(|j| {
                        let c = [
                            self.intersections[i][j],
                            self.intersections[i][j + 1],
                            self.intersections[i + 1][j],
                            self.intersections[i + 1][j + 1],
                        ];
                        (
                            c.iter().map(|p| p.0).sum::<f32>() / 4.0,
                            c.iter().map(|p| p.1).sum::<f32>() / 4.0,
                        )
                    })
                    .collect()
            })
            .collect()
    }

    fn draw(&self, img: &mut opencv::core::Mat) -> opencv::Result<()> {
        // Green circles at intersection corners.
        for row in &self.intersections {
            for &(x, y) in row {
                circle(
                    img,
                    Point::new(x as i32, y as i32),
                    5,
                    VecN::new(0.0, 255.0, 0.0, 1.0),
                    1,
                    LINE_AA,
                    0,
                )?;
            }
        }
        // Blue filled dots at cell centers.
        for row in self.cell_centers() {
            for (x, y) in row {
                circle(
                    img,
                    Point::new(x as i32, y as i32),
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
}

// ---------------------------------------------------------------------------
// Grid detection
// ---------------------------------------------------------------------------

/// Intersection of two Hough-polar lines: x·cos θ + y·sin θ = ρ.
fn intersect_polar(rho1: f32, theta1: f32, rho2: f32, theta2: f32) -> (f32, f32) {
    // Cramer's rule on the 2×2 system.
    let theta1 = (theta1 - 0.5 * PI) % TAU;
    let theta2 = (theta2 - 0.5 * PI) % TAU;
    let det = theta1.cos() * theta2.sin() - theta1.sin() * theta2.cos();
    let x = (rho1 * theta2.sin() - rho2 * theta1.sin()) / det;
    let y = (theta1.cos() * rho2 - theta2.cos() * rho1) / det;
    (x, y)
}

/// Absolute angular difference, wrapping at TAU.
fn angle_diff(a: f32, b: f32) -> f32 {
    let d = (a - b).abs() % TAU;
    d.min(TAU - d)
}

fn detect_grid(detected: &[rubik_vision::grid::Line]) -> Option<Grid> {
    let thetas: Vec<f32> = detected.iter().map(|l| l.theta).collect();
    let [t1, t2] = find_two_peaks(&thetas, THETA_BANDWIDTH)?;

    let rhos1: Vec<f32> = detected
        .iter()
        .filter(|l| angle_diff(l.theta, t1) < THETA_TOLERANCE)
        .map(|l| l.rho)
        .collect();
    let rhos2: Vec<f32> = detected
        .iter()
        .filter(|l| angle_diff(l.theta, t2) < THETA_TOLERANCE)
        .map(|l| l.rho)
        .collect();

    let mut peaks1 = find_peaks(&rhos1, RHO_BANDWIDTH, 0.3);
    let mut peaks2 = find_peaks(&rhos2, RHO_BANDWIDTH, 0.3);

    if peaks1.is_empty() || peaks2.is_empty() {
        return None;
    }

    peaks1.sort_by(|a, b| a.partial_cmp(b).unwrap());
    peaks2.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let intersections = peaks1
        .iter()
        .map(|&r1| {
            peaks2
                .iter()
                .map(|&r2| intersect_polar(r1, t1, r2, t2))
                .collect()
        })
        .collect();

    Some(Grid {
        theta1: t1,
        theta2: t2,
        _rhos1: peaks1,
        _rhos2: peaks2,
        intersections,
    })
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

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
        let name = path.file_name().unwrap().to_string_lossy();

        let mut img = imread(path.to_str().unwrap(), IMREAD_COLOR).unwrap();
        let detected = lines(&mut img).unwrap();

        match detect_grid(&detected) {
            None => println!("{name}: no grid detected"),
            Some(grid) => {
                let cells = (grid.rows().saturating_sub(1), grid.cols().saturating_sub(1));
                println!(
                    "{name}: {}×{} cells, lines at {:.1}° / {:.1}°",
                    cells.0,
                    cells.1,
                    grid.theta1.to_degrees(),
                    grid.theta2.to_degrees(),
                );
                grid.draw(&mut img).unwrap();
            }
        }

        imwrite(out_img.to_str().unwrap(), &img, &Vector::new()).unwrap();
    }
}
