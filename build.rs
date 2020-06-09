use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;
use std::fs::File;
use std::io::{Read, Write};
use yaml_rust::{YamlLoader, YamlEmitter, Yaml, yaml};
use proc_use::UseBuilder;
use man_dmenu::*;

fn main() {
    let out_path    = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_path = PathBuf::from("target").join(env::var("PROFILE").unwrap());

    // First, figure out what plugins we are using
    let plugins_str = env::var("PLUGINS")
	.expect("\n\n\
		 ┌─────────────────────────────────┐\n\
		 │               BUILD FAILED                │\n\
		 │PLUGINS environment variable not found.    │\n\
		 │Help: You should call make instead of cargo│\n\
		 └─────────────────────────────────┘\
		 \n\n");
    let plugins: Vec<&str> =
	if plugins_str.len() > 0 {
	    plugins_str
		.split(" ").collect()
	} else {
	    Vec::new()
	};

    // Next, set up overrider/proc_use/clap for the source files
    let mut watch_globs = Vec::new();
    println!("cargo:rerun-if-changed=src/dmenu/cli_base.yml");
    let mut cli_base = File::open("src/dmenu/cli_base.yml").unwrap();
    let mut yaml_str = String::new();
    if let Err(err) = cli_base.read_to_string(&mut yaml_str) {
	panic!("Could not read yaml base file {}", err);	
    }
    yaml_str = yaml_str.replace("$VERSION", &env::var("CARGO_PKG_VERSION").unwrap());

    // prepare to edit cli_base args
    let mut yaml = &mut YamlLoader::load_from_str(&yaml_str).unwrap()[0];
    let yaml_args: &mut Vec<yaml::Yaml> = get_yaml_args(&mut yaml);

    // For every plugin, check if it has arguements. If so, add them to clap and overrider
    // While we're here, set proc_use to watch the plugin entry points
    for plugin in plugins {
	let plugin_file = format!("src/plugins/{}/plugin.yml", plugin);
	println!("cargo:rerun-if-changed={}", plugin_file);
	let mut plugin_base = File::open(plugin_file).unwrap();
	let mut plugin_yaml_str = String::new();
	if let Err(err) = plugin_base.read_to_string(&mut plugin_yaml_str) {
	    panic!("Could not read yaml base file {}", err);	
	}
	let mut plugin_yaml = &mut YamlLoader::load_from_str(&plugin_yaml_str).unwrap()[0];
	let plugin_yaml_args: &mut Vec<yaml::Yaml> = get_yaml_args(&mut plugin_yaml);

	yaml_args.append(plugin_yaml_args);

	watch_globs.push((
	    format!("src/plugins/{}/{}", plugin, get_yaml_about(plugin_yaml)),
	    format!("plugin_{}", plugin)
	));
    }

    // Now that cli is built, generate manpage
    let mut manpage = Manpage::new("dmenu", &env::var("CARGO_PKG_VERSION").unwrap(), 1);
    manpage.desc_short("dynamic menu")
	.description("dmenu",
		     "is a dynamic menu for X, which reads a list of newline\\-separated \
		      items from stdin.  When the user selects an item and presses \
		      Return, their choice is printed to stdout and dmenu terminates.  \
		      Entering text will narrow the items to those matching the tokens \
		      in the input."
	).description("dmenu_run",
		      "is a script used by\n\
		       .IR dwm (1)\n\
		       which lists programs in the user's $PATH and runs the result in \
		       their $SHELL.");

    manpage.arg(Some('c'), Some("aa"), vec!["1", "2"], "ree");
    manpage.arg(None, Some("ee"), vec!["1"], "bigger ree");

    manpage.write_to_file(target_path.join("dmenu.1"));

    // Dump yaml, clap will parse this later.
    let mut yaml_out = String::new();
    let mut emitter = YamlEmitter::new(&mut yaml_out);
    emitter.dump(yaml).unwrap();
    let mut cli_finished_file = File::create(out_path.join("cli.yml")).unwrap();
    if let Err(err) = cli_finished_file.write_all(yaml_out.as_bytes()) {
	panic!("Could not write generated yaml file to OUT_DIR: {}", err);
    }

    // finalize overrider and proc_use initilization
    let mut usebuilder = UseBuilder::new();
    let mut overrider_watch = vec!["src/dmenu/plugin_entry.rs"];
    for file in &watch_globs {
	overrider_watch.push(&file.0);
	usebuilder.mod_glob_alias(&file.0, &file.1);
    }

    // Write overrider and proc_use
    overrider_build::watch_files(overrider_watch);
    usebuilder
	.write_to_file_all(out_path.join("proc_mod_plugin.rs"));



    
    // Next order of business:
    // servo-fontconfig does a good job for 99% of fontconfig,
    // but doesn't quite get everything we need.
    // So, generate bindings here.
    let mut builder_main = bindgen::Builder::default();
    builder_main = builder_main.header("src/headers/fontconfig.h");

    if cfg!(feature = "Xinerama") {
	println!("cargo:rustc-link-lib=Xinerama");
	builder_main = builder_main.header("src/headers/xinerama.h");
    }

    builder_main.parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings_main")
        .write_to_file(out_path.join("bindings_main.rs"))
        .expect("Couldn't write bindings_main!");

    // Additionally, the x11 crate doesn't null terminate its strings for some
    //   strange reason, so a bit of extra work is required
    bindgen::Builder::default()
	.header("src/headers/xlib.h")
	.ignore_functions() // strip out unused and warning-prone functions
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings_xlib")
        .write_to_file(out_path.join("bindings_xlib.rs"))
        .expect("Couldn't write bindings_xlib!");
    
    // Because bindings depend on files in the headers directory,
    // we want to rebuild on edit
    for e in WalkDir::new("src/headers").into_iter().filter_map(|e| e.ok()) {
        if e.metadata().unwrap().is_file() {
	    let name = e.path().to_str().unwrap();
	    if name.as_bytes()[name.len()-1] != '~' as u8 { // ignore editor files
		println!("cargo:rerun-if-changed={}", e.path().display());
	    }
	}
    }

    // Finally, link the libs we just generated bindings for
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xft");

    // Additionally, config.mk controlls lots of fun things, so rebuild on change
    println!("cargo:rerun-if-changed=config.mk");
}


// util functions below

fn get_yaml_about(yaml: &mut Yaml) -> &mut String {
    match yaml {
	Yaml::Hash(hash) => {
	    for field in hash {
		if let Yaml::String(fieldname) = field.0 {
		    if fieldname == "entry" {
			match field.1 {
			    Yaml::String(arr) => {
				return arr;
			    },
			    _ => panic!("Incorrect arg format on cli_base"),
			}
		    }
		}
	    }
	},
	_ => panic!("Incorrect yaml format on cli_base"),
    }
    panic!("No args found in yaml object");
}

fn get_yaml_args(yaml: &mut Yaml) -> &mut Vec<yaml::Yaml> {
    match yaml {
	Yaml::Hash(hash) => {
	    for field in hash {
		if let Yaml::String(fieldname) = field.0 {
		    if fieldname == "args" {
			match field.1 {
			    Yaml::Array(arr) => {
				return arr;
			    },
			    _ => panic!("Incorrect arg format on cli_base"),
			}
		    }
		}
	    }
	},
	_ => panic!("Incorrect yaml format on cli_base"),
    }
    panic!("No args found in yaml object");
}
