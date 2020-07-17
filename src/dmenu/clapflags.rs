use clap::{ArgMatches, App};
use itertools::Itertools;
use yaml_rust::yaml::Yaml;
use regex::RegexBuilder;

use crate::config::{Clrs::*, Schemes::*, Config, DefaultWidth};
use crate::result::*;

lazy_static::lazy_static! {
    static ref YAML: Yaml = {
        clap::YamlLoader::load_from_str(include_str!(concat!(env!("BUILD_DIR"), "/cli.yml")))
            .expect("failed to load YAML file") 
            .pop()
            .unwrap()
        };
    pub static ref CLAP_FLAGS: ArgMatches<'static> = App::from_yaml(&YAML).get_matches();     
}

pub fn validate(config: &mut Config) -> CompResult<()> {

    if CLAP_FLAGS.occurrences_of("version") > 2 {
	eprintln!("More than 2 version flags do nothing special");
    }
    if CLAP_FLAGS.occurrences_of("version") == 1 {
	return Die::stdout(format!("dmenu-rs {}", env!("VERSION")));
    }
    if CLAP_FLAGS.occurrences_of("version") >= 2 {
	let plugins = env!("PLUGINS");
	if plugins.len() == 0 {
	    return Die::stdout(format!("dmenu-rs {}\n\
					Compiled with rustc {}\n\
					Compiled without plugins",
				       env!("VERSION"),
				       rustc_version_runtime::version(),
	    ));
	} else {
	    return Die::stdout(format!("dmenu-rs {}\n\
					Compiled with rustc {}\n\
					Compiled with plugins:\n\
					{}",
				       env!("VERSION"),
				       rustc_version_runtime::version(),
				       plugins.split(" ")
				       .map(|p| format!("- {}", p))
				       .join("\n"),
	    ));
	}
    }
    
    let color_regex = RegexBuilder::new("^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})\0$")
	.case_insensitive(true)
	.build().map_err(|_| Die::Stderr("Could not build regex"
					 .to_owned()))?;

    // bottom
    if CLAP_FLAGS.occurrences_of("bottom") == 1 {
	config.topbar = false;
    }

    // fast
    if CLAP_FLAGS.occurrences_of("fast") == 1 {
	config.fast = true;
    }

    // insensitive
    if CLAP_FLAGS.occurrences_of("insensitive") == 1 {
	config.case_sensitive = false;
    }

    // lines
    if let Some(lines) = CLAP_FLAGS.value_of("lines") {
	config.lines = lines.parse::<u32>()
	    .map_err(|_| Die::Stderr("-l: Lines must be a non-negaitve integer"
				     .to_owned()))?;
    }

    // monitor
    if let Some(monitor) = CLAP_FLAGS.value_of("monitor") {
	config.mon = monitor.parse::<i32>()
	    .map_err(|_| Die::Stderr("-m: Monitor must be a non-negaitve integer"
				     .to_owned()))?;
    }

    // prompt
    if let Some(prompt) = CLAP_FLAGS.value_of("prompt") {
	config.prompt = prompt.to_string();
    }

    // font
    if let Some(fonts) = CLAP_FLAGS.values_of("font") {
	let default = config.fontstrings.pop().unwrap();
	config.fontstrings = fonts.map(|f| f.to_string()).collect();
	config.fontstrings.push(default);
    }

    // color_normal_background
    if let Some(color) = CLAP_FLAGS.value_of("color_normal_background") {
	let mut color = color.to_string();
	color.push('\0');
	color_regex.find_iter(&color).nth(0)
	    .ok_or(Die::Stderr("--nb: Color must be in hex format (#123456 or #123)"
			       .to_owned()))?;
	config.colors[SchemeNorm as usize][ColBg as usize]
	    .copy_from_slice(color.as_bytes());
    }

    // color_normal_foreground
    if let Some(color) = CLAP_FLAGS.value_of("color_normal_foreground") {
	let mut color = color.to_string();
	color.push('\0');
	color_regex.find_iter(&color).nth(0)
	    .ok_or(Die::Stderr("--nf: Color must be in hex format (#123456 or #123)"
			       .to_owned()))?;
	config.colors[SchemeNorm as usize][ColFg as usize]
	    .copy_from_slice(color.as_bytes());
    }

    // color_selected_background
    if let Some(color) = CLAP_FLAGS.value_of("color_selected_background") {
	let mut color = color.to_string();
	color.push('\0');
	color_regex.find_iter(&color).nth(0)
	    .ok_or(Die::Stderr("--sb: Color must be in hex format (#123456 or #123)"
			       .to_owned()))?;
	config.colors[SchemeSel as usize][ColBg as usize]
	    .copy_from_slice(color.as_bytes());
    }

    // color_selected_foreground
    if let Some(color) = CLAP_FLAGS.value_of("color_selected_foreground") {
	let mut color = color.to_string();
	color.push('\0');
	color_regex.find_iter(&color).nth(0)
	    .ok_or(Die::Stderr("--sf: Color must be in hex format (#123456 or #123)"
			       .to_owned()))?;
	config.colors[SchemeSel as usize][ColFg as usize]
	    .copy_from_slice(color.as_bytes());
    }

    // window
    if let Some(window) = CLAP_FLAGS.value_of("window") {
	config.embed = window.parse::<u64>()
	    .map_err(|_| Die::Stderr("-w: Window ID must be a valid X window ID string"
				     .to_owned()))?;
    }

    // nostdin
    if CLAP_FLAGS.occurrences_of("nostdin") == 1 {
	config.nostdin = true;
    }

    // render_minheight
    if let Some(minheight) = CLAP_FLAGS.value_of("render_minheight") {
	config.render_minheight = minheight.parse::<u32>()
	    .map_err(|_| Die::Stderr("--render_minheight: Height must be an integet number of \
				  pixels".to_owned()))?;
    }

    // render_overrun
    if CLAP_FLAGS.occurrences_of("render_overrun") == 1 {
	config.render_overrun = true;
	config.render_flex = true;
    }

    // render_flex
    if CLAP_FLAGS.occurrences_of("render_flex") == 1 {
	config.render_flex = true;
    }

    // render_rightalign
    if CLAP_FLAGS.occurrences_of("render_rightalign") == 1 {
	config.render_rightalign = true;
    }
    
    // render_default_width
    if let Some(arg) = CLAP_FLAGS.value_of("render_default_width") {
	if !arg.contains("=") {
	    config.render_default_width = match arg {
		"min" => DefaultWidth::Min,
		"items" => DefaultWidth::Items,
		"max" => {
		    config.render_rightalign = true;
		    DefaultWidth::Max
		},
		_ => return Die::stderr("--render_default_width: invalid arguement".to_owned()),
	    }
	} else {
	    let vec: Vec<&str> = arg.split("=").collect();
	    if vec.len() != 2 || (vec.len() > 0 && vec[0] != "custom") {
		return Die::stderr("Incorrect format for --render_default_width, \
				    see help for details".to_owned());
	    }
	    let width = vec[1].parse::<u8>();
	    if width.is_err() || *width.as_ref().unwrap() > 100 {
		return Die::stderr("--render_default_width: custom width \
				      must be a positive integer".to_owned());
	    }
	    config.render_default_width = DefaultWidth::Custom(width.unwrap());
	}
    }

    Ok(())
}
