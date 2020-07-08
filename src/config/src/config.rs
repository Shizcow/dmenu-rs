use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use man_dmenu::*;
use std::path::PathBuf;
use itertools::Itertools;
use yaml_rust::{YamlEmitter, Yaml, yaml};

mod util;
use crate::util::*;

fn main() {
    let target_path = PathBuf::from(env!("BUILD_TARGET_PATH"));
    let build_path = PathBuf::from(env!("BUILD_PATH"));
    
    // First, figure out what plugins we are using
    let plugins = get_selected_plugin_list();

    // Next, set up the following for plugin files:
    // 1) clap command line yaml file
    // 2) proc_use import files
    // 3) overrider watch files
    // 4) Cargo.toml<dmenu-build> plugin dependencies
    let mut watch_globs = Vec::new();
    let mut deps_vec = Vec::new();
    
    // prepare to edit cli_base args
    let mut yaml = get_yaml("../dmenu/cli_base.yml");
    let yaml_args: &mut Vec<yaml::Yaml> = get_yaml_args(&mut yaml).unwrap();

    // For every plugin, check if it has arguements. If so, add them to clap and overrider
    // While we're here, set proc_use to watch the plugin entry points
    for plugin in plugins {
	let mut plugin_yaml = get_yaml(&format!("../plugins/{}/plugin.yml", plugin));
	
	if let Some(plugin_yaml_args) = get_yaml_args(&mut plugin_yaml) {
	    yaml_args.append(plugin_yaml_args);
	}

	watch_globs.push((
	    format!("../plugins/{}/{}", plugin, get_yaml_top_level(&mut plugin_yaml, "entry")
		    .expect("No args found in yaml object")),
	    format!("plugin_{}", plugin)
	));

	if let Some(deps_name) = get_yaml_top_level(&mut plugin_yaml, "cargo_dependencies") {
	    let deps_file = format!("../plugins/{}/{}", plugin, deps_name);
	    let mut deps_base = File::open(deps_file).unwrap();
	    let mut deps_read_str = String::new();
	    if let Err(err) = deps_base.read_to_string(&mut deps_read_str) {
		panic!("Could not read dependency base file {}", err);	
	    }
	    deps_vec.push(deps_read_str);
	}

	if let Some(build_command) = get_yaml_top_level(&mut plugin_yaml, "build") {
	    let mut command = Command::new("sh");
	    let dir = format!("../plugins/{}/", plugin);
	    command.current_dir(dir);
	    let output = command.arg("-c")
		.arg(build_command).output()
		.expect("failed to execute plugin build command");
	    if output.status.success() {
		println!("Plugin '{}' build command success.\n\
		      - stdout: \n{}\n\
		      - stderr: \n{}", plugin,
		     String::from_utf8_lossy(&output.stdout),
		     String::from_utf8_lossy(&output.stderr));
	    } else {
		panic!("Plugin '{}' build command failed.\n\
			- stdout: '{}'\n\
			- stderr: '{}'", plugin,
		       String::from_utf8_lossy(&output.stdout),
		       String::from_utf8_lossy(&output.stderr));
	    }
	}
    }

    // Write additional dependency list
    let mut deps_finished_file = File::create(build_path.join("deps.toml")).unwrap();
    if let Err(err) = deps_finished_file.write_all(deps_vec.join("\n").as_bytes()) {
	panic!("Could not write generated dependency file to OUT_DIR: {}", err);
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

    for arg in yaml_args {
	let hash = match arg {
	    Yaml::Hash(hash) => {
		hash
	    },
	    _ => panic!("yaml arg must be hash"),
	};
	let keys: Vec<_> = hash.keys().cloned().collect();
	let mut short = None;
	let mut long = None;
	let mut help = None;
	let mut inputs = Vec::new();
	match hash.get(&keys[0]) {
	    Some(Yaml::Hash(hash)) => {
		let keys: Vec<_> = hash.keys().cloned().collect();
		for key in &keys {
		    let keyname = match &key {
			Yaml::String(string) => string,
			_ => panic!("yaml arg name must be string"),
		    };
		    let keyvalue = 
			match hash.get(key) {
			    Some(Yaml::String(string)) => {
				string
			    },
			    _ => continue,
			};
		    if keyname == "long_help" {
			help = Some(keyvalue);
		    } else if keyname == "help" && help.is_none() {
			help = Some(keyvalue);
		    } else if keyname == "short" {
			short = Some(keyvalue);
		    } else if keyname == "long" {
			long = Some(keyvalue);
		    } else if keyname == "value_name" {
			inputs = vec![keyvalue.clone()];
		    } else if keyname == "value_names" {
			inputs = keyvalue.split(" ").map(|c| c.to_string()).collect();
		    }
		}
	    },
	    _ => panic!("Invalid yaml format"),
	}
	if short.is_some() || long.is_some() {
	    manpage.arg(short.map(|s| s.chars().nth(0).unwrap()),
			long.map(|s| s.to_string()), inputs,
			help.expect("yaml: help must be provided")
			.to_string());
	}
    }

    manpage.write_to_file(target_path.join("dmenu.1"));

    // Dump yaml, clap will parse this later.
    let mut yaml_out = String::new();
    let mut emitter = YamlEmitter::new(&mut yaml_out);
    emitter.dump(&mut yaml).unwrap();
    let mut cli_finished_file = File::create(build_path.join("cli.yml")).unwrap();
    if let Err(err) = cli_finished_file.write_all(yaml_out.as_bytes()) {
	panic!("Could not write generated yaml file to OUT_DIR: {}", err);
    }

    // dump plugin watch files to target/build so src/build/build.rs can pick up on them
    let watch_indicator_string = watch_globs.into_iter().map(
	|(glob, alias)|
	format!("{}\n{}\n", glob, alias)).join("\n");
    let mut watch_indicator_file = File::create(build_path.join("watch_files")).unwrap();
    if let Err(err) = watch_indicator_file.write_all(watch_indicator_string.as_bytes()) {
	panic!("Could not write generated watch file to OUT_DIR: {}", err);
    }
}
