#![allow(unused)]
use x11::xft::*;
use x11::xlib::*;
use x11::xrender::*;
use std::ptr;
use std::ffi::{CString};
use std::os::raw::{c_uchar};
use libc::{setlocale, LC_CTYPE, c_int};
use std::mem::{self, MaybeUninit};
use std::thread::sleep;
use std::time::Duration;

mod drw;
use drw::{Drw, Run};
mod globals;
use globals::*;
mod config;
use config::*;
mod additional_bindings;
mod item;
use item::Items;
mod util;
mod fnt;
mod run;


fn main() {    
    let mut config = Config::default();
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
	    if (pseudo_globals.embed == 0) {
		root
	    } else {
		pseudo_globals.embed
	    };
	XGetWindowAttributes(dpy, parentwin, &mut wa); // will non-gracefully panic on fail with a decent error message
	let mut drw = Drw::new(dpy, screen, root, wa, pseudo_globals, config);

	// TODO: OpenBSD

	drw.setup(parentwin, root);

	drw.run();
    }
}
