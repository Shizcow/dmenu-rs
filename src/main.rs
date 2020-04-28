#![allow(unused)]
use x11::xft::*;
use x11::xlib::*;
use x11::xrender::*;
use std::ptr;
use std::ffi::{CString};
use std::os::raw::{c_uchar};
use libc::{setlocale, LC_CTYPE};

use std::mem::{self, MaybeUninit};

mod drw;
use drw::{Drw, PseudoGlobals};
mod config;
use config::*;

fn main() {
    let default_font = CString::new("monospace:size=10").unwrap().into_raw();
    let fonts = vec![default_font]; // TODO: move into config
    let embed = 0; // TODO

    let config = Config::default();

    let pseudo_globals = PseudoGlobals::default();
    
    unsafe {
	let mut wa: XWindowAttributes = MaybeUninit::uninit().assume_init();//<_>::uninit_alloc();
	// TODO: command line arguements
	if (setlocale(LC_CTYPE, ptr::null())==ptr::null_mut() || XSupportsLocale()==0) {
	    eprintln!("warning: no locale support\n");
	}
	let dpy = XOpenDisplay(ptr::null_mut());
	if (dpy==ptr::null_mut()) {
	    panic!("cannot open display");
	}
	let screen = XDefaultScreen(dpy);
	let root = XRootWindow(dpy, screen);

	let parentwin =
	    if (embed == 0) {
		root
	    } else {
		embed
	    };
	XGetWindowAttributes(dpy, parentwin, &mut wa); // will non-gracefully panic on fail with a decent error message
	let mut drw = Drw::new(dpy, screen, root, wa, pseudo_globals);
	if(!drw.fontset_create(fonts)) {
	    panic!("no fonts could be loaded.");
	}

	drw.setup(config, parentwin);


	println!("{:?}", drw);
    }
}
