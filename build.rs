use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    // We need access to several of the #defines in fontconfig.h, so generate bindings for them here
    // The rest of the autogenerated bindings aren't suitable, so we use the servo-fontconfig crate
    let mut bindings_builder = bindgen::Builder::default();
    bindings_builder = bindings_builder.header("headers/fontconfig.h");

    if cfg!(feature = "Xinerama") {
	println!("cargo:rustc-link-lib=Xinerama");
	bindings_builder = bindings_builder.header("headers/xinerama.h");
    }
    println!("cargo:rustc-link-lib=X11");
    //println!("cargo:rustc-link-lib=Xrender");
    println!("cargo:rustc-link-lib=Xft");

    let bindings = bindings_builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    

    let mut bindings_builder2 = bindgen::Builder::default();
    bindings_builder2 = bindings_builder2.header("headers/xlib.h");
    let bindings2 = bindings_builder2
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    let out_path2 = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings2
        .write_to_file(out_path2.join("xlib.rs"))
        .expect("Couldn't write bindings!");

    
    Command::new("gcc").args(&["/home/notroot/Downloads/xorg-libX11/src/xlibi18n/ICWrap.c",
			       "-c", "-g", "-o"])
        .arg(&format!("{}/ICWrap.o", env::var("OUT_DIR").unwrap()))
        .status().unwrap();
    Command::new("ar").args(&["crus", "libICWrap.a", "ICWrap.o"])
        .current_dir(&Path::new(&env::var("OUT_DIR").unwrap()))
        .status().unwrap();
    Command::new("gcc").args(&["/home/notroot/Downloads/xorg-libX11/src/reallocarray.c",
			       "-c", "-g", "-o"])
        .arg(&format!("{}/Xreallocarray.o", env::var("OUT_DIR").unwrap()))
        .status().unwrap();
    Command::new("ar").args(&["crus", "libXreallocarray.a", "Xreallocarray.o"])
        .current_dir(&Path::new(&env::var("OUT_DIR").unwrap()))
        .status().unwrap();
    println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-lib=static=ICWrap");
    println!("cargo:rustc-link-lib=static=Xreallocarray");
    println!("cargo:rerun-if-changed=/home/notroot/Downloads/xorg-libX11/src/xlibi18n/ICWrap.c");
    
    // bindings depend on files in the headers directory, so make sure they are tracked for rebuild on edit
    for e in WalkDir::new("headers").into_iter().filter_map(|e| e.ok()) {
        if e.metadata().unwrap().is_file() {
	    let name = e.path().to_str().unwrap();
	    if name.as_bytes()[name.len()-1] != '~' as u8 { // ignore editor files
		println!("cargo:rerun-if-changed={}", e.path().display());
	    }
	}
    }

}
