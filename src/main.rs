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
use regex::RegexBuilder;

use drw::Drw;
use globals::*;
use config::{*, Clrs::*, Schemes::*};

fn main() {    
    let mut config = Config::default();
    let pseudo_globals = PseudoGlobals::default();
    let color_regex = RegexBuilder::new("^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})\0$")
					    .case_insensitive(true)
					    .build()
					    .expect("Could not build regex");
    let mut args = std::env::args().skip(1);  // skip filename
    
    unsafe {

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
		    if let Some(mut val) = args.next() {
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
			    "-p" => // adds prompt to left of input field
				config.prompt = val,
			    "-fn" => // font or font set
				config.default_font = val,
			    c @ "-nb" | c @ "-nf" | c @ "-sb" | c @ "-sf" => {
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
				    panic!("Color must be in hex format (#123456 or #123)");
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
