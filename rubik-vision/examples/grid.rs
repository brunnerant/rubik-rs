use std::ffi::OsStr;

use opencv::core::Vector;
use opencv::imgcodecs::{IMREAD_COLOR, imread, imwrite};
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
        let mut img = imread(img.to_str().unwrap(), IMREAD_COLOR).unwrap();
        let lines = lines(&mut img).unwrap();
        imwrite(out.to_str().unwrap(), &img, &Vector::new()).unwrap();
    }
}
