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

use x11::xlib::*;
use std::ptr;
use libc::{setlocale, LC_CTYPE};
use std::mem::MaybeUninit;
use regex::RegexBuilder;
#[cfg(target_os = "openbsd")]
use pledge;

use drw::Drw;
use globals::*;
use config::{*, Clrs::*, Schemes::*};

fn main() { // just a wrapper to ensure a clean death in the event of error
    std::process::exit(match start() {
	Ok(_) => 0,
	Err(err) => {
	    if err.len() > 0 {
		eprintln!("{}", err);
	    }
	    1
	},
    });
}

fn start() -> Result<(), String> { 
    let mut config = Config::default();
    let pseudo_globals = PseudoGlobals::default();
    let color_regex = match RegexBuilder::new("^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})\0$")
	.case_insensitive(true)
	.build() {
	    Ok(re) => re,
	    Err(_) => return Err(format!("Could not build regex")),
	};
    let mut args = std::env::args().skip(1);  // skip filename
    
    unsafe {

	while let Some(arg) = args.next() {
	    match arg.as_str() {
		// These options take no arguements
		"-v" | "--version" => { // prints version information (and exit)
		    println!("dmenu-{}", env!("CARGO_PKG_VERSION"));
		    return Ok(());
		},
		"-b" => // appears at the bottom of the screen
		    config.topbar = false,
		"-f" => // grabs keyboard before reading stdin
		    config.fast = false,
		"-i" => // case-insensitive item matching
		    config.case_sensitive = false,
		// these options take two arguements
		flag => {
		    match (flag, args.next()) {
			("-l", Some(val)) => { // number of lines in vertical list
			    match val.parse::<u32>() {
				Ok(lines) => config.lines = lines,
				_ => {
				    return Err(format!("-l: Lines must be a non-negaitve integer"));
				},
			    }
			},
			("-m", Some(val)) => { // monitor to place menu on
			    match val.parse::<i32>() {
				Ok(monitor) if monitor >= 0 => config.mon = monitor,
				_ => {
				    return Err(format!("-m: Monitor must be a non-negaitve integer"));
				},
			    }
			},
			("-p", Some(val)) => // adds prompt to left of input field
			    config.prompt = val,
			("-fn", Some(val)) => // font or font set
			    config.default_font = val,
			("-nb", Some(mut val)) => {
			    val.push('\0');
			    match color_regex.find_iter(&val).nth(0) {
				Some(_) => config.colors[SchemeNorm as usize][ColBg as usize].copy_from_slice(val.as_bytes()),
				None => return Err(format!("-nb: Color must be in hex format (#123456 or #123)")),
			    }
			},
			("-nf", Some(mut val)) => {
			    val.push('\0');
			    match color_regex.find_iter(&val).nth(0) {
				Some(_) => config.colors[SchemeNorm as usize][ColFg as usize].copy_from_slice(val.as_bytes()),
				None => return Err(format!("-nb: Color must be in hex format (#123456 or #123)")),
			    }
			},
			("-sb", Some(mut val)) => {
			    val.push('\0');
			    match color_regex.find_iter(&val).nth(0) {
				Some(_) => config.colors[SchemeSel as usize][ColBg as usize].copy_from_slice(val.as_bytes()),
				None => return Err(format!("-nb: Color must be in hex format (#123456 or #123)")),
			    }
			},
			("-sf", Some(mut val)) => {
			    val.push('\0');
			    match color_regex.find_iter(&val).nth(0) {
				Some(_) => config.colors[SchemeSel as usize][ColFg as usize].copy_from_slice(val.as_bytes()),
				None => return Err(format!("-nb: Color must be in hex format (#123456 or #123)")),
			    }
			},
			("-w", Some(val)) => { // embedding window id
			    match val.parse::<u64>() {
				Ok(id) => config.embed = id,
				_ => {
				    return Err(format!("-w: Window ID must be a valid X window ID string"));
				},
			    }
			},
			_ => {
			    return Err(format!("{}\n{}",
					       "usage: dmenu [-bfiv] [-l lines] \
						[-p prompt] [-fn font] [-m monitor]",
					       "             [-nb color] [-nf color] \
						[-sb color] [-sf color] [-w windowid]"));
			},
		    }
		},
	    }
	}
	
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

	return match Drw::new(dpy, screen, root, wa, pseudo_globals, config) {
	    Ok(mut drw) => {
		if cfg!(target_os = "openbsd") {
		    if let Err(_) = pledge::pledge("stdio rpath", None) {
			return Err(format!("Could not pledge"));
		    }
		}
		
		if let Err(err) = drw.setup(parentwin, root) {
		    return Err(err);
		}

		drw.run()
	    },
	    Err(err) => Err(err),
	}
    }
}
