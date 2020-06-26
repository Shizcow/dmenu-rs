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
#[cfg(target_os = "openbsd")]
use pledge;

use drw::Drw;
use globals::*;
use config::*;

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

    clapflags::validate(&mut config)?;
    
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
	    pledge::pledge("stdio rpath", None)
		.map_err(|_| format!("Could not pledge"))?;
	}
	
	drw.setup(parentwin, root)?;
	drw.run()
    }
}
