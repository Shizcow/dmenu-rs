use yaml_rust::{YamlLoader, Yaml, yaml};
use std::fs::File;
use std::io::Read;
use std::env;

pub fn get_selected_plugin_list() -> Vec<String> {
    let plugins_str = env::var("PLUGINS")
	.expect("\n\n\
		 ┌─────────────────────────────────┐\n\
		 │               BUILD FAILED                │\n\
		 │PLUGINS environment variable not found.    │\n\
		 │Help: You should call make instead of cargo│\n\
		 └─────────────────────────────────┘\
		 \n\n");
    if plugins_str.len() > 0 {
	plugins_str
	    .split(" ").map(|s| s.to_string()).collect()
    } else {
	Vec::new()
    }
}

pub fn get_yaml(file: &str) -> Yaml {
    let mut base = File::open(file).expect(file);
    let mut yaml_str = String::new();
    if let Err(err) = base.read_to_string(&mut yaml_str) {
	panic!("Could not read yaml base file {}", err);	
    }
    yaml_str = yaml_str.replace("$VERSION", &env!("VERSION"));
    YamlLoader::load_from_str(&yaml_str).unwrap().swap_remove(0)
}

#[allow(unused)]
pub fn get_yaml_top_level<'a>(yaml: &'a mut Yaml, fieldsearch: &str) -> Option<&'a mut String> {
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

#[allow(unused)]
pub fn get_yaml_args(yaml: &mut Yaml) -> &mut Vec<yaml::Yaml> {
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
