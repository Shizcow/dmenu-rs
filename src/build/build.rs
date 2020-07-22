use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use proc_use::UseBuilder;

#[path = "../config/src/util.rs"]
mod util;

fn main() {
    let build_path_str = "../../target/build";
    
    let out_path   = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_path = PathBuf::from(build_path_str);
    println!("cargo:rustc-env=BUILD_DIR={}", build_path_str);

    println!("cargo:rerun-if-changed={}", build_path.join("watch_files").canonicalize().unwrap().display().to_string());
    println!("cargo:rerun-if-env-changed=PLUGINS");
    
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

    // if plugin files are changed without modifying anything else,
    // sometimes overrider needs to be ran again
    let plugins = util::get_selected_plugin_list();
    for plugin in plugins{
	let mut plugin_yaml = util::get_yaml(&format!("../plugins/{}/plugin.yml", plugin), Some(&plugin));
	println!("cargo:rerun-if-changed=../plugins/{}/{}", plugin, util::get_yaml_top_level(&mut plugin_yaml, "entry").unwrap());
    }

    // link libs
    if cfg!(feature = "Xinerama") {
	println!("cargo:rustc-link-lib=Xinerama");
    }
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xft");
}
