mod util;
mod drw;
mod globals;
mod config;
mod additional_bindings;
mod item;
mod fnt;
mod init;
mod setup;
mod run;
mod clapflags;
mod plugin_entry;
mod plugins {
    include!(concat!(env!("OUT_DIR"), "/proc_mod_plugin.rs"));
}

use x11::xlib::*;
use std::ptr;
use libc::{setlocale, LC_CTYPE};
use std::mem::MaybeUninit;
use regex::RegexBuilder;
#[cfg(target_os = "openbsd")]
use pledge;

use drw::Drw;
use globals::*;
use config::*;

use clapflags::*;

fn main() { // just a wrapper to ensure a clean death in the event of error
    std::process::exit(match try_main() {
	Ok(_) => 0,
	Err(err) => {
	    if err.len() > 0 {
		eprintln!("Unrecoverable error: {}", err);
	    }
	    1
	},
    });
}

fn try_main() -> Result<(), String> {
    let mut config = Config::default();
    let pseudo_globals = PseudoGlobals::default();
    let color_regex = match RegexBuilder::new("^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})\0$")
	.case_insensitive(true)
	.build() {
	    Ok(re) => re,
	    Err(_) => return Err(format!("Could not build regex")),
	};

    if CLAP_FLAGS.occurrences_of("bottom") == 1 {
	config.topbar = false;
    }
    if CLAP_FLAGS.occurrences_of("fast") == 1 {
	config.fast = true;
    }
    if CLAP_FLAGS.occurrences_of("insensitive") == 1 {
	config.case_sensitive = false;
    }

    if let Some(lines) = CLAP_FLAGS.value_of("lines") {
	match lines.parse::<u32>() {
	    Ok(lines) => config.lines = lines,
	    _ => {
		return Err(format!("-l: Lines must be a non-negaitve integer"));
	    },
	}
    }
    if let Some(monitor) = CLAP_FLAGS.value_of("monitor") {
	match monitor.parse::<i32>() {
	    Ok(monitor) if monitor >= 0 => config.mon = monitor,
	    _ => {
		return Err(format!("-m: Monitor must be a non-negaitve integer"));
	    },
	}
    }
    if let Some(prompt) = CLAP_FLAGS.value_of("prompt") {
	config.prompt = prompt.to_string();
    }
    if let Some(font) = CLAP_FLAGS.value_of("font") {
	config.default_font = font.to_string();
    }
    if let Some(color) = CLAP_FLAGS.value_of("color_normal_background") {
	let mut color = color.to_string();
	color.push('\0');
	match color_regex.find_iter(&color).nth(0) {
	    Some(_) => config.colors[SchemeNorm as usize][ColBg as usize]
		.copy_from_slice(color.as_bytes()),
	    None => return Err(format!("-nb: Color must be in hex format (#123456 or #123)")),
	}
    }
    if let Some(color) = CLAP_FLAGS.value_of("color_normal_foreground") {
	let mut color = color.to_string();
	color.push('\0');
	match color_regex.find_iter(&color).nth(0) {
	    Some(_) => config.colors[SchemeNorm as usize][ColFg as usize]
		.copy_from_slice(color.as_bytes()),
	    None => return Err(format!("-nb: Color must be in hex format (#123456 or #123)")),
	}
    }
    if let Some(color) = CLAP_FLAGS.value_of("color_selected_background") {
	let mut color = color.to_string();
	color.push('\0');
	match color_regex.find_iter(&color).nth(0) {
	    Some(_) => config.colors[SchemeSel as usize][ColBg as usize]
		.copy_from_slice(color.as_bytes()),
	    None => return Err(format!("-nb: Color must be in hex format (#123456 or #123)")),
	}
    }
    if let Some(color) = CLAP_FLAGS.value_of("color_selected_foreground") {
	let mut color = color.to_string();
	color.push('\0');
	match color_regex.find_iter(&color).nth(0) {
	    Some(_) => config.colors[SchemeSel as usize][ColFg as usize]
		.copy_from_slice(color.as_bytes()),
	    None => return Err(format!("-nb: Color must be in hex format (#123456 or #123)")),
	}
    }
    if let Some(window) = CLAP_FLAGS.value_of("window") {
	match window.parse::<u64>() {
	    Ok(id) => config.embed = id,
	    _ => {
		return Err(format!("-w: Window ID must be a valid X window ID string"));
	    },
	}
    }
    
    unsafe {	
	if setlocale(LC_CTYPE, ptr::null())==ptr::null_mut() || XSupportsLocale()==0 {
	    return Err(format!("warning: no locale support"));
	}
	let dpy = XOpenDisplay(ptr::null_mut());
	if dpy==ptr::null_mut() {
	    return Err(format!("cannot open display"));
	}
	let screen = XDefaultScreen(dpy);
	let root = XRootWindow(dpy, screen);
	let parentwin = root.max(config.embed);
	let mut wa: XWindowAttributes = MaybeUninit::uninit().assume_init();
	XGetWindowAttributes(dpy, parentwin, &mut wa);

	let mut drw = Drw::new(dpy, screen, root, wa, pseudo_globals, config)?;
	if cfg!(target_os = "openbsd") {
	    if let Err(_) = pledge::pledge("stdio rpath", None) {
		return Err(format!("Could not pledge"));
	    }
	}
	
	drw.setup(parentwin, root)?;
	drw.run()
    }
}
