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
mod result;
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
use result::*;

fn main() { // just a wrapper to ensure a clean death in the event of error
    std::process::exit(match try_main() {
	Ok(_) => 0,
	Err(Die::Stdout(msg)) => {
	    if msg.len() > 0 {
		println!("{}", msg)
	    }
	    0
	},
	Err(Die::Stderr(msg)) => {
	    if msg.len() > 0 {
		eprintln!("{}", msg)
	    }
	    1
	},
    });
}

fn try_main() -> CompResult<()> {
    let mut config = Config::default();
    let pseudo_globals = PseudoGlobals::default();

    clapflags::validate(&mut config)?;
    
    unsafe {	
	if setlocale(LC_CTYPE, ptr::null())==ptr::null_mut() || XSupportsLocale()==0 {
	    return Die::stderr("warning: no locale support".to_owned());
	}
	let dpy = XOpenDisplay(ptr::null_mut());
	if dpy==ptr::null_mut() {
	    return Die::stderr("cannot open display".to_owned());
	}
	let screen = XDefaultScreen(dpy);
	let root = XRootWindow(dpy, screen);
	let parentwin = root.max(config.embed);
	let mut wa: XWindowAttributes = MaybeUninit::uninit().assume_init();
	XGetWindowAttributes(dpy, parentwin, &mut wa);

	let mut drw = Drw::new(dpy, screen, root, wa, pseudo_globals, config)?;
	if cfg!(target_os = "openbsd") {
	    pledge::pledge("stdio rpath", None)
		.map_err(|_| Die::Stderr("Could not pledge".to_owned()))?;
	}
	
	drw.setup(parentwin, root)?;
	drw.run()
    }
}
