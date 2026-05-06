use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let tdlib_dir = PathBuf::from(&manifest_dir).join("tdlib");

    println!(
        "cargo:rustc-link-search=native={}",
        tdlib_dir.display()
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // OUT_DIR is something like target/debug/build/<pkg>/out — go up 3 levels to get target/debug/
    let target_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("unexpected OUT_DIR structure")
        .to_path_buf();

    for entry in fs::read_dir(&tdlib_dir).expect("tdlib/ directory missing") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("dll") {
            let dest = target_dir.join(path.file_name().unwrap());
            fs::copy(&path, &dest).unwrap_or_else(|e| {
                panic!("Failed to copy {:?} → {:?}: {}", path, dest, e)
            });
        }
    }
}
