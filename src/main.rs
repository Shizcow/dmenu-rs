#![allow(unused)]
use x11::xft::*;
use x11::xlib::*;
use x11::xrender::*;
use std::ptr;
use std::ffi::{CString};
use std::os::raw::{c_uchar};
use libc::{setlocale, LC_CTYPE, isatty, c_int};
use std::mem::{self, MaybeUninit};
use std::time::Duration;
use std::thread::sleep;
use std::io::{self, BufRead};

mod drw;
use drw::{Drw, PseudoGlobals};
mod config;
use config::*;
mod additional_bindings;
mod item;
use item::Item;
mod xlib_additional;


fn readstdin(drw: &mut Drw, config: &mut Config) -> Vec<Item> {
    let mut imax = 0;
    let items: Vec<Item> = io::stdin().lock().lines().enumerate().map(|line_enum|{
	match line_enum.1 {
	    Ok(line) => {
		let (width, _) = drw.font_getexts(&drw.fonts[0], line.as_ptr(), line.len() as c_int);
		if width as i32 > config.inputw {
		    config.inputw = width as i32;
		    imax = line_enum.0;
		}
		Item::new(line, 0)
	    },
	    Err(_) => panic!("Could not read from stdin"),
	}
    }).collect();
    config.inputw = drw.fontset_getwidth(&items[imax].text) + (3/4)*drw.pseudo_globals.lrpad;
    items
}

fn grabkeyboard(dpy: *mut Display, embed: bool) {
    let ts = Duration::from_millis(1);

    if (embed) {
	return;
    }
    /* try to grab keyboard, we may have to wait for another process to ungrab */
    for _ in 0..1000 {
	if unsafe{XGrabKeyboard(dpy, XDefaultRootWindow(dpy), True, GrabModeAsync,
				  GrabModeAsync, CurrentTime) == GrabSuccess} {
	    return;
	}
	sleep(ts);
    }
    panic!("cannot grab keyboard");
}

fn main() {
    let default_font = CString::new("monospace:size=10").unwrap().into_raw();
    let fonts = vec![default_font]; // TODO: move into config
    let embed = 0; // TODO
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
	drw.pseudo_globals.lrpad = drw.fonts[0].height as i32;

	// TODO: OpenBSD

	let stdin_text = Vec::new();
	/*
	if (fast && isatty(0) == 0) {
	    grabkeyboard(drw.dpy, false); // TODO: embed
	    stdin_text = readstdin(&mut drw, &mut config);
	} else {
	    stdin_text = readstdin(&mut drw, &mut config);
	    grabkeyboard(drw.dpy, false); // TODO: embed
	}*/

	drw.setup(config, parentwin, root, stdin_text);


	println!("{:?}", drw);
    }
}
