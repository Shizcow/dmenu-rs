use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use proc_use::UseBuilder;
use walkdir::WalkDir;

fn main() {
    let build_path_str = "../../target/build";
    
    let out_path   = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_path = PathBuf::from(build_path_str);
    println!("cargo:rustc-env=BUILD_DIR={}", build_path_str);
    
    // grab the list of plugins and aliases
    let mut plugin_file = File::open(build_path.join("watch_files")).unwrap();
    let mut plugin_str = String::new();
    if let Err(err) = plugin_file.read_to_string(&mut plugin_str) {
	panic!("Could not read plugin file {}", err);
    }

    let mut lines = plugin_str.split("\n");
    let mut watch_globs = Vec::new();
    
    while let (Some(path), Some(alias), Some("")) = (lines.next(), lines.next(), lines.next()) {
	watch_globs.push((path, alias));
    }
    
    // finalize overrider and proc_use initilization
    let mut usebuilder = UseBuilder::new();
    let mut overrider_watch = vec!["../dmenu/plugin_entry.rs"];
    for file in &watch_globs {
	overrider_watch.push(&file.0);
	usebuilder.mod_glob_alias(&file.0, &file.1);
    }

    // Write overrider and proc_use
    overrider_build::watch_files(overrider_watch);
    usebuilder
	.write_to_file_all(out_path.join("proc_mod_plugin.rs"));

    // link libs
    if cfg!(feature = "Xinerama") {
	println!("cargo:rustc-link-lib=Xinerama");
    }
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xft");

    // watch files
    for dir in &["../headers", build_path_str] {
	for e in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if e.metadata().unwrap().is_file() {
		let name = e.path().to_str().unwrap();
		if name.as_bytes()[name.len()-1] != '~' as u8 { // ignore editor files
		    println!("cargo:rerun-if-changed={}", e.path().display());
		}
	    }
	}
    }
}
