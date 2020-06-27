use clap::{ArgMatches, App};
use yaml_rust::yaml::Yaml;
use regex::RegexBuilder;

use crate::config::{Clrs::*, Schemes::*, Config, DefaultWidth};


lazy_static::lazy_static! {
    static ref YAML: Yaml = {
        clap::YamlLoader::load_from_str(include_str!(concat!(env!("BUILD_DIR"), "/cli.yml")))
            .expect("failed to load YAML file") 
            .pop()
            .unwrap()
        };
    pub static ref CLAP_FLAGS: ArgMatches<'static> = App::from_yaml(&YAML).get_matches();     
}

pub fn validate(config: &mut Config) -> Result<(), String> {
    
    let color_regex = RegexBuilder::new("^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})\0$")
	.case_insensitive(true)
	.build().map_err(|_| format!("Could not build regex"))?;

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
	    .map_err(|_| format!("-l: Lines must be a non-negaitve integer"))?;
    }

    // monitor
    if let Some(monitor) = CLAP_FLAGS.value_of("monitor") {
	config.mon = monitor.parse::<i32>()
	    .map_err(|_| format!("-m: Monitor must be a non-negaitve integer"))?;
    }

    // prompt
    if let Some(prompt) = CLAP_FLAGS.value_of("prompt") {
	config.prompt = prompt.to_string();
    }

    // font
    if let Some(font) = CLAP_FLAGS.value_of("font") {
	config.default_font = font.to_string();
    }

    // color_normal_background
    if let Some(color) = CLAP_FLAGS.value_of("color_normal_background") {
	let mut color = color.to_string();
	color.push('\0');
	color_regex.find_iter(&color).nth(0)
	    .ok_or(format!("--nb: Color must be in hex format (#123456 or #123)"))?;
	config.colors[SchemeNorm as usize][ColBg as usize]
	    .copy_from_slice(color.as_bytes());
    }

    // color_normal_foreground
    if let Some(color) = CLAP_FLAGS.value_of("color_normal_foreground") {
	let mut color = color.to_string();
	color.push('\0');
	color_regex.find_iter(&color).nth(0)
	    .ok_or(format!("--nf: Color must be in hex format (#123456 or #123)"))?;
	config.colors[SchemeNorm as usize][ColFg as usize]
	    .copy_from_slice(color.as_bytes());
    }

    // color_selected_background
    if let Some(color) = CLAP_FLAGS.value_of("color_selected_background") {
	let mut color = color.to_string();
	color.push('\0');
	color_regex.find_iter(&color).nth(0)
	    .ok_or(format!("--sb: Color must be in hex format (#123456 or #123)"))?;
	config.colors[SchemeSel as usize][ColBg as usize]
	    .copy_from_slice(color.as_bytes());
    }

    // color_selected_foreground
    if let Some(color) = CLAP_FLAGS.value_of("color_selected_foreground") {
	let mut color = color.to_string();
	color.push('\0');
	color_regex.find_iter(&color).nth(0)
	    .ok_or(format!("--sf: Color must be in hex format (#123456 or #123)"))?;
	config.colors[SchemeSel as usize][ColFg as usize]
	    .copy_from_slice(color.as_bytes());
    }

    // window
    if let Some(window) = CLAP_FLAGS.value_of("window") {
	config.embed = window.parse::<u64>()
	    .map_err(|_| format!("-w: Window ID must be a valid X window ID string"))?;
    }

    // nostdin
    if CLAP_FLAGS.occurrences_of("nostdin") == 1 {
	config.nostdin = true;
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
		_ => return Err(format!("--render_default_width: invalid arguement")),
	    }
	} else {
	    let vec: Vec<&str> = arg.split("=").collect();
	    if vec.len() != 2 || (vec.len() > 0 && vec[0] != "custom") {
		return Err(format!("Incorrect format for --render_default_width, \
				    see help for details"));
	    }
	    let width = vec[1].parse::<u8>();
	    if width.is_err() || *width.as_ref().unwrap() > 100 {
		return Err(format!("--render_default_width: custom width \
				      must be a positive integer"));
	    }
	    config.render_default_width = DefaultWidth::Custom(width.unwrap());
	}
    }

    Ok(())
}
