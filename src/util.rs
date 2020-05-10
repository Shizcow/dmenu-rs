use x11::xlib::{CurrentTime, RevertToParent, XSetInputFocus, XGetInputFocus, Window, Display,
		GrabSuccess, GrabModeAsync, True, XDefaultRootWindow, XGrabKeyboard};
use crate::drw::Drw;
use libc::c_int;
use crate::item::Item;
use crate::config::Config;
use std::mem::{self, MaybeUninit};
use std::time::Duration;
use std::thread::sleep;
use std::io::{self, BufRead};

pub fn readstdin(drw: &mut Drw) -> Vec<Item> {
    let items: Vec<Item> = io::stdin().lock().lines().enumerate().map(|line_enum|{
	match line_enum.1 {
	    Ok(line) => {
		let item = Item::new(line, false, drw);
		if item.width as i32 > drw.pseudo_globals.inputw {
		    drw.pseudo_globals.inputw = item.width as i32;
		}
		item
	    },
	    Err(_) => panic!("Could not read from stdin"),
	}
    }).collect();
    items
}

pub fn grabkeyboard(dpy: *mut Display, embed: Window) {
    let ts = Duration::from_millis(1);

    if (embed == 0) {
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

pub fn grabfocus(drw: &Drw) {
    unsafe {
    let ts = Duration::from_millis(1);
    let mut focuswin: Window = MaybeUninit::uninit().assume_init();
    let mut revertwin = MaybeUninit::uninit().assume_init();

    for _ in 0..100 {
	XGetInputFocus(drw.dpy, &mut focuswin, &mut revertwin);
	if focuswin == drw.pseudo_globals.win {
	    return;
	}
	XSetInputFocus(drw.dpy, drw.pseudo_globals.win, RevertToParent, CurrentTime);
	sleep(ts);
    }
	panic!("cannot grab focus");
    }
}
