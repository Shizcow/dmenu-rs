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
    
    // servo-fontconfig does a good job for 99% of fontconfig,
    // but doesn't quite get everything we need.
    // So, generate bindings here.
    let mut builder_main = bindgen::Builder::default();
    builder_main = builder_main.header("src/fontconfig.h");
    builder_main = builder_main.header("src/xinerama.h");

    builder_main.parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings_main")
        .write_to_file(build_path.join("bindings_main.rs"))
        .expect("Couldn't write bindings_main!");

    // Additionally, the x11 crate doesn't null terminate its strings for some
    //   strange reason, so a bit of extra work is required
    bindgen::Builder::default()
	.header("src/xlib.h")
	.ignore_functions() // strip out unused and warning-prone functions
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings_xlib")
        .write_to_file(build_path.join("bindings_xlib.rs"))
        .expect("Couldn't write bindings_xlib!");
}
