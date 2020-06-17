use std::env;
use std::fs::File;
use std::io::{Read, Write};
use yaml_rust::{YamlLoader, YamlEmitter, Yaml, yaml};
use man_dmenu::*;
use std::path::PathBuf;
use itertools::Itertools;

fn main() {
    let target_path = PathBuf::from(env!("BUILD_TARGET_PATH"));
    let build_path = PathBuf::from(env!("BUILD_PATH"));
    
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

    // Next, set up the following for plugin files:
    // 1) clap command line yaml file
    // 2) proc_use import files
    // 3) overrider watch files
    // 4) Cargo.toml<dmenu-build> plugin dependencies
    let mut watch_globs = Vec::new();
    let mut deps_vec = Vec::new();
    let mut cli_base = File::open("../dmenu/cli_base.yml").unwrap();
    let mut yaml_str = String::new();
    if let Err(err) = cli_base.read_to_string(&mut yaml_str) {
	panic!("Could not read yaml base file {}", err);	
    }
    yaml_str = yaml_str.replace("$VERSION", &env!("VERSION"));

    // prepare to edit cli_base args
    let mut yaml = &mut YamlLoader::load_from_str(&yaml_str).unwrap()[0];
    let yaml_args: &mut Vec<yaml::Yaml> = get_yaml_args(&mut yaml);

    // For every plugin, check if it has arguements. If so, add them to clap and overrider
    // While we're here, set proc_use to watch the plugin entry points
    for plugin in plugins {
	let plugin_file = format!("../plugins/{}/plugin.yml", plugin);
	let mut plugin_base = File::open(plugin_file).unwrap();
	let mut plugin_yaml_str = String::new();
	if let Err(err) = plugin_base.read_to_string(&mut plugin_yaml_str) {
	    panic!("Could not read yaml base file {}", err);	
	}
	let mut plugin_yaml = &mut YamlLoader::load_from_str(&plugin_yaml_str).unwrap()[0];
	let plugin_yaml_args: &mut Vec<yaml::Yaml> = get_yaml_args(&mut plugin_yaml);

	yaml_args.append(plugin_yaml_args);

	watch_globs.push((
	    format!("../plugins/{}/{}", plugin, get_yaml_top_level(plugin_yaml, "entry")
		    .expect("No args found in yaml object")),
	    format!("plugin_{}", plugin)
	));

	if let Some(deps_name) = get_yaml_top_level(plugin_yaml, "cargo_dependencies") {
	    let deps_file = format!("../plugins/{}/{}", plugin, deps_name);
	    let mut deps_base = File::open(deps_file).unwrap();
	    let mut deps_read_str = String::new();
	    if let Err(err) = deps_base.read_to_string(&mut deps_read_str) {
		panic!("Could not read dependency base file {}", err);	
	    }
	    deps_vec.push(deps_read_str);
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
		    if keyname == "help" {
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
    emitter.dump(yaml).unwrap();
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



// util functions below

fn get_yaml_top_level<'a>(yaml: &'a mut Yaml, fieldsearch: &str) -> Option<&'a mut String> {
    match yaml {
	Yaml::Hash(hash) => {
	    for field in hash {
		if let Yaml::String(fieldname) = field.0 {
		    if fieldname == fieldsearch {
			match field.1 {
			    Yaml::String(arr) => {
				return Some(arr);
			    },
			    _ => panic!("Incorrect arg format on cli_base"),
			}
		    }
		}
	    }
	},
	_ => panic!("Incorrect yaml format on cli_base"),
    }
    None
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
