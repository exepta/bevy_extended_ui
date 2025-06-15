use std::fs;
use std::path::Path;

fn main() {
    let _ = std::env::var("OUT_DIR").unwrap();
    let target_assets = Path::new("../../assets/extended_ui/icons");
    fs::create_dir_all(&target_assets).unwrap();
    fs::copy("assets/extended_ui/icons/check-mark.png", target_assets.join("check-mark.png")).unwrap();
}