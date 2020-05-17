use x11::xlib::{XCreateGC, XCreatePixmap, XSetLineAttributes, XDefaultDepth, XDefaultColormap,
		XDefaultVisual, JoinMiter, CapButt, LineSolid, XWindowAttributes,
		Window, Display};
use x11::xft::{XftColorAllocName, XftColor};
use libc::{c_char, c_int, isatty};
use std::{mem::{MaybeUninit, ManuallyDrop}, ffi::CStr, ptr};

use crate::drw::Drw;
use crate::config::{Config, Schemes::*};
use crate::item::Items;
use crate::util::*;
use crate::globals::*;
use crate::fnt::*;

impl Drw {
    pub fn new(dpy: *mut Display, screen: c_int, root: Window, wa: XWindowAttributes, pseudo_globals: PseudoGlobals, config: Config) -> Result<Self, ()> {
	unsafe {
	    let drawable = XCreatePixmap(dpy, root, wa.width as u32, wa.height as u32, XDefaultDepth(dpy, screen) as u32);
	    let gc = XCreateGC(dpy, root, 0, ptr::null_mut());
	    XSetLineAttributes(dpy, gc, 1, LineSolid, CapButt, JoinMiter);
	    let mut ret = Self{wa, dpy, screen, root, drawable, gc, fonts: Vec::new(),
			       pseudo_globals, config,
			       scheme: MaybeUninit::uninit().assume_init(),
			       w: MaybeUninit::uninit().assume_init(),
			       h: MaybeUninit::uninit().assume_init(),
			       input: "".to_string(),
			       items: {MaybeUninit::uninit()}.assume_init()};

	    for j in 0..SchemeLast as usize {
		if let Ok(scheme) = ret.scm_create(ret.config.colors[j]) {
		    ret.pseudo_globals.schemeset[j] = scheme;
		} else {
		    return Err(());
		}
	    }

	    
	    if !ret.fontset_create(vec![ret.config.default_font.as_ptr() as *mut i8]) {
		eprintln!("no fonts could be loaded.");
		return Err(());
	    }
	    ret.pseudo_globals.lrpad = ret.fonts[0].height as i32;

	    
	    ret.items = ManuallyDrop::new(Items::new(
		if ret.config.fast && isatty(0) == 0 {
		    grabkeyboard(ret.dpy, ret.config.embed);
		    match readstdin(&mut ret) {
			Ok(items) => items,
			Err(_) => return Err(()),
		    }
		} else {
		    let tmp = match readstdin(&mut ret) {
			Ok(items) => items,
			Err(_) => return Err(()),
		    };
		    grabkeyboard(ret.dpy, ret.config.embed);
		    tmp
		}));

	    ret.config.lines = ret.config.lines.min(ret.items.data.len() as u32);
	    
	    Ok(ret)
	}
    }

    fn scm_create(&self, clrnames: [[u8; 8]; 2]) -> Result<[*mut XftColor; 2], ()> {
	let ret: [*mut XftColor; 2] = unsafe{
	    [
		Box::into_raw(Box::new(MaybeUninit::uninit().assume_init())),
		Box::into_raw(Box::new(MaybeUninit::uninit().assume_init())),
	    ]
	};
	if let Err(_) = self.clr_create(ret[0], clrnames[0].as_ptr() as *const c_char) {
	    return Err(());
	};
	if let Err(_) = self.clr_create(ret[1], clrnames[1].as_ptr() as *const c_char) {
	    return Err(());
	};
	Ok(ret)
    }

    fn clr_create(&self, dest: *mut XftColor, clrname: *const c_char) -> Result<(), ()> {
	unsafe {
	    if clrname == ptr::null_mut() { // TODO: statically prove this isn't required
		return Err(());
	    }
	    if XftColorAllocName(self.dpy, XDefaultVisual(self.dpy, self.screen), XDefaultColormap(self.dpy, self.screen), clrname, dest) == 0 {
		eprintln!("error, cannot allocate color {:?}", CStr::from_ptr(clrname));
		Err(())
	    } else {
		Ok(())
	    }
	}
    }

    fn fontset_create(&mut self, fonts: Vec<*mut c_char>) -> bool {
	if fonts.len() == 0 {
	    return false;
	}

	for font in fonts.into_iter().rev() {
	    let to_push = Fnt::new(self, font, ptr::null_mut());
	    if to_push.is_some() {
		self.fonts.push(to_push.unwrap());
	    } else {
		return false;
	    }
	}

	true
    }
}
