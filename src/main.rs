#![allow(unused)]
use x11::xft::*;
use x11::xlib::*;
use x11::xrender::*;
use std::ptr;
use std::ffi::{CString};
use std::os::raw::{c_uchar};
use libc::{setlocale, LC_CTYPE, isatty, c_int};
use std::mem::{self, MaybeUninit};
use std::thread::sleep;
use std::time::Duration;

mod drw;
use drw::Drw;
mod globals;
use globals::*;
mod config;
use config::*;
mod additional_bindings;
mod item;
use item::Items;
mod util;
use util::{readstdin, grabkeyboard};
mod fnt;


fn main() {
    let default_font = CString::new("monospace:size=10").unwrap().into_raw();
    let fonts = vec![default_font]; // TODO: move into config
    let mut config = Config::default();
    let pseudo_globals = PseudoGlobals::default();
    let fast = false;
    
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
	if(!drw.fontset_create(fonts)) {
	    panic!("no fonts could be loaded.");
	}
	drw.pseudo_globals.lrpad = drw.fonts[0].height as i32;

	// TODO: OpenBSD

	let items = Items::new(
	if (fast && isatty(0) == 0) {
	    grabkeyboard(drw.dpy, drw.pseudo_globals.embed); // TODO: embed
	    readstdin(&mut drw)
	} else {
	    let tmp = readstdin(&mut drw);
	    grabkeyboard(drw.dpy, drw.pseudo_globals.embed); // TODO: embed
	    tmp
	});

	drw.setup(parentwin, root, items);


	
	sleep(Duration::from_millis(1000));
	println!("{:?}", drw);
    }
}
