use x11::xlib::{CurrentTime, RevertToParent, XSetInputFocus, XGetInputFocus, Window, Display,
		GrabSuccess, GrabModeAsync, True, XDefaultRootWindow, XGrabKeyboard};
use crate::drw::Drw;
use crate::item::Item;
use std::mem::MaybeUninit;
use std::time::Duration;
use std::thread::sleep;
use std::io::{self, BufRead};
use crate::result::*;

pub fn readstdin(drw: &mut Drw) -> CompResult<Vec<Item>> {
    let mut lines: Vec<String> = Vec::new();
    for line in io::stdin().lock().lines() {
	match line {
	    Ok(l) => lines.push(l),
	    Err(e) => return Die::stderr(format!("Could not read from stdin: {}", e)),
	}
    }
    let mut ret = Vec::new();
    for line in drw.format_stdin(lines)?.into_iter() {
	let item = Item::new(line, false, drw)?;
	if item.width as i32 > drw.pseudo_globals.inputw {
	    drw.pseudo_globals.inputw = item.width as i32;
	}
	ret.push(item)
    }
    Ok(ret)
}

pub fn grabkeyboard(dpy: *mut Display, embed: Window) -> CompResult<()> {
    let ts = Duration::from_millis(1);

    if embed != 0 {
	return Ok(());
    }
    /* try to grab keyboard, we may have to wait for another process to ungrab */
    for _ in 0..1000 {
	if unsafe{XGrabKeyboard(dpy, XDefaultRootWindow(dpy), True, GrabModeAsync,
				GrabModeAsync, CurrentTime) == GrabSuccess} {
	    return Ok(());
	}
	sleep(ts);
    }
    Die::stderr("cannot grab keyboard".to_owned())
}

pub fn grabfocus(drw: &Drw) -> CompResult<()> {
    unsafe {
	let ts = Duration::from_millis(1);
	let mut focuswin: Window = MaybeUninit::uninit().assume_init();
	let mut revertwin = MaybeUninit::uninit().assume_init();

	for _ in 0..100 {
	    XGetInputFocus(drw.dpy, &mut focuswin, &mut revertwin);
	    if focuswin == drw.pseudo_globals.win {
		return Ok(());
	    }
	    XSetInputFocus(drw.dpy, drw.pseudo_globals.win, RevertToParent, CurrentTime);
	    sleep(ts);
	}
	Die::stderr("cannot grab focus".to_owned())
    }
}
