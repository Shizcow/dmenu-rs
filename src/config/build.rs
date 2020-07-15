use std::path::PathBuf;

// bindgen is pretty slow, so we add a layer of indirection,
// making sure it's only ran when needed. build.rs has great
// support for that, so here it is
fn main() {
    let mut target_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    target_path.pop();
    target_path.pop();
    target_path = target_path.join("target");
    let build_path = target_path.join("build");

    println!("cargo:rustc-env=BUILD_TARGET_PATH={}", target_path.display().to_string());
    println!("cargo:rustc-env=BUILD_PATH={}", build_path.display().to_string());
}
