mod drw;
mod globals;
mod config;
mod additional_bindings;
mod item;
mod util;
mod fnt;
mod init;
mod setup;
mod run;

use x11::xlib::*;
use std::ptr;
use libc::{setlocale, LC_CTYPE};
use std::mem::MaybeUninit;

use drw::Drw;
use globals::*;
use config::*;

fn main() {    
    let mut config = Config::default();
    let pseudo_globals = PseudoGlobals::default();
    
    unsafe {
	let mut args = std::env::args().skip(1);  // skip filename

	while let Some(arg) = args.next() {
	    match arg.as_str() {
		// These arguements take no arguements
		"-v" => // prints version information (and exit)
		    return println!("dmenu-{}", env!("CARGO_PKG_VERSION")),
		"-b" => // appears at the bottom of the screen
		    config.topbar = false,
		"-f" => // grabs keyboard before reading stdin
		    config.fast = false,
		"-i" => // case-insensitive item matching
		    config.case_sensitive = false,
		// these options take two arguements
		flag => {
		    if let Some(val) = args.next() {
			match flag {
			    "-l" => {
				if let Ok(lines) = val.parse::<u32>() {
				    config.lines = lines;
				} else {
				    panic!("-l: Lines must be a positive integer");
				}
			    },
			    _ => panic!("Usage"),
			}
		    }
		},
	    }
	}
	
	// TODO: command line arguements
	if setlocale(LC_CTYPE, ptr::null())==ptr::null_mut() || XSupportsLocale()==0 {
	    eprintln!("warning: no locale support\n");
	}
	let dpy = XOpenDisplay(ptr::null_mut());
	if dpy==ptr::null_mut() {
	    panic!("cannot open display");
	}
	let screen = XDefaultScreen(dpy);
	let root = XRootWindow(dpy, screen);
	let parentwin = root.max(config.embed);
	let mut wa: XWindowAttributes = MaybeUninit::uninit().assume_init();
	XGetWindowAttributes(dpy, parentwin, &mut wa); // will non-gracefully panic on fail with a decent error message
	let mut drw = Drw::new(dpy, screen, root, wa, pseudo_globals, config);

	// TODO: OpenBSD

	drw.setup(parentwin, root);

	drw.run();
    }
}
