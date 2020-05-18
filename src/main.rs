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

use drw::Drw;
use globals::*;
use config::{*, Clrs::*, Schemes::*};

fn main() -> Result<(), ()> {    
    let mut config = Config::default();
    let pseudo_globals = PseudoGlobals::default();
    let color_regex = RegexBuilder::new("^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})\0$")
					    .case_insensitive(true)
					    .build()
					    .expect("Could not build regex");
    let mut args = std::env::args().skip(1);  // skip filename
    
    unsafe {

	while let Some(arg) = args.next() {
	    match arg.as_str() {
		// These arguements take no arguements
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
				    eprintln!("-l: Lines must be a non-negaitve integer");
				    return Err(());
				},
			    }
			},
			("-m", Some(val)) => { // monitor to place menu on
			    match val.parse::<i32>() {
				Ok(monitor) if monitor >= 0 => config.mon = monitor,
				_ => {
				    eprintln!("-m: Monitor must be a non-negaitve integer");
				    return Err(());
				},
			    }
			},
			("-p", Some(val)) => // adds prompt to left of input field
			    config.prompt = val,
			("-fn", Some(val)) => // font or font set
			    config.default_font = val,
			(c @ "-nb", Some(mut val))
			    | (c @ "-nf", Some(mut val))
			    | (c @ "-sb", Some(mut val))
			    | (c @ "-sf", Some(mut val)) => {
			    val.push('\0');
			    if color_regex.find_iter(&val).nth(0).is_some() {
				config.colors[if c.as_bytes()[1] == 'n' as u8 {
				    SchemeNorm // -nb or -nf => normal scheme
				} else {
				    SchemeSel // -sb or -sf => selected scheme
				} as usize][if c.as_bytes()[2] == 'b' as u8 {
				    ColBg // -nb or -sb => background color
				} else {
				    ColFg // -nf or -sf => foreground color
				} as usize][..val.len()]
				    .copy_from_slice(val.as_bytes());
			    } else {
				eprintln!("{}: Color must be in hex format (#123456 or #123)", c);
				    return Err(());
			    }
			},
			("-w", Some(val)) => { // embedding window id
			    match val.parse::<u64>() {
				Ok(id) => config.embed = id,
				_ => {
				    eprintln!("-w: Window ID must be a valid X window ID string");
				    return Err(());
				},
			    }
			},
			_ => {
			    eprintln!("{}\n{}",
				      "usage: dmenu [-bfiv] [-l lines] \
				       [-p prompt] [-fn font] [-m monitor]",
				      "             [-nb color] [-nf color] \
				       [-sb color] [-sf color] [-w windowid]");
				    return Err(());
			},
		    }
		},
	    }
	}
	
	if setlocale(LC_CTYPE, ptr::null())==ptr::null_mut() || XSupportsLocale()==0 {
	    eprintln!("warning: no locale support\n");
	    return Err(());
	}
	let dpy = XOpenDisplay(ptr::null_mut());
	if dpy==ptr::null_mut() {
	    eprintln!("cannot open display");
	    return Err(());
	}
	let screen = XDefaultScreen(dpy);
	let root = XRootWindow(dpy, screen);
	let parentwin = root.max(config.embed);
	let mut wa: XWindowAttributes = MaybeUninit::uninit().assume_init();
	XGetWindowAttributes(dpy, parentwin, &mut wa);

	return match Drw::new(dpy, screen, root, wa, pseudo_globals, config) {
	    Ok(mut drw) => {
		// TODO: OpenBSD

		if drw.setup(parentwin, root).is_err() {
		    return Err(());
		}

		drw.run()
	    },
	    Err(_) => Err(()),
	}
    }
}
