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

	// TODO: gracefull exit/die (include return for dealloc)
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
			    "-l" => { // number of lines in vertical list
				match val.parse::<u32>() {
				    Ok(lines) => config.lines = lines,
				    _ => panic!("-l: Lines must be a non-negaitve integer"),
				}
			    },
			    "-m" => { // monitor to place menu on
				match val.parse::<i32>() {
				    Ok(monitor) if monitor >= 0 => config.mon = monitor,
				    _ => panic!("-m: Monitor must be a non-negaitve integer"),
				}
			    },
			    "-p" => { // adds prompt to left of input field
				config.prompt = val;
			    },
			    "-fn" => { // font or font set
				config.default_font = val;
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
