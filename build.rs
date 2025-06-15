use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let source_dir = PathBuf::from(manifest_dir).join("assets/extended_ui/icons");
    
    let target_dir = PathBuf::from("../assets/extended_ui/icons");
    
    fs::create_dir_all(&target_dir).expect("Failed to create target assets directory");
    
    for entry in fs::read_dir(&source_dir).expect("Failed to read source asset dir") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            let target_path = target_dir.join(file_name);
            fs::copy(&path, &target_path)
                .expect(&format!("Failed to copy asset to {:?}", target_path));
        }
    }

    println!("cargo:rerun-if-changed=assets/extended_ui/icons");
}