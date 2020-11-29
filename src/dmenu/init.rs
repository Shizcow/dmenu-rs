use x11::xlib::{XCreateGC, XCreatePixmap, XSetLineAttributes, XDefaultDepth, XDefaultColormap,
		XDefaultVisual, JoinMiter, CapButt, LineSolid, XWindowAttributes,
		Window, Display};
use x11::xft::{XftColorAllocName, XftColor};
use libc::{c_char, c_int, isatty};
use std::{mem::MaybeUninit, ffi::CStr, ptr};

use crate::drw::Drw;
use crate::config::{Config, Schemes::*};
use crate::item::Items;
use crate::util::*;
use crate::globals::*;
use crate::fnt::*;
use crate::result::*;

impl Drw {
    pub fn new(dpy: *mut Display, screen: c_int, root: Window, wa: XWindowAttributes, pseudo_globals: PseudoGlobals, config: Config) -> CompResult<Self> {
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
			       items: None};
	    
	    ret.fontset_create()?;
	    ret.pseudo_globals.lrpad = ret.fonts[0].height as i32;
	    
	    ret.items = if ret.config.nostdin {
		ret.format_stdin(vec![])?;
		grabkeyboard(ret.dpy, ret.config.embed)?;
		Some(Items::new(Vec::new()))
	    } else {Some(Items::new(
		if ret.config.fast && isatty(0) == 0 {
		    grabkeyboard(ret.dpy, ret.config.embed)?;
		    readstdin(&mut ret)?
		} else {
		    let tmp = readstdin(&mut ret)?;
		    grabkeyboard(ret.dpy, ret.config.embed)?;
		    tmp
		}))
	    };
	    
	    for j in 0..SchemeLast as usize {
		ret.pseudo_globals.schemeset[j] = ret.scm_create(ret.config.colors[j])?;
	    }

	    ret.config.lines = ret.config.lines.min(ret.get_items().len() as u32);

	    
	    Ok(ret)
	}
    }

    fn scm_create(&self, clrnames: [[u8; 8]; 2]) -> CompResult<[*mut XftColor; 2]> {
	let ret: [*mut XftColor; 2] = unsafe {
	    [
		Box::into_raw(Box::new(MaybeUninit::uninit().assume_init())),
		Box::into_raw(Box::new(MaybeUninit::uninit().assume_init())),
	    ]
	};
	self.clr_create(ret[0], clrnames[0].as_ptr() as *const c_char)?;
	self.clr_create(ret[1], clrnames[1].as_ptr() as *const c_char)?;
	Ok(ret)
    }

    fn clr_create(&self, dest: *mut XftColor, clrname: *const c_char) -> CompResult<()> {
	unsafe {
	    if XftColorAllocName(self.dpy, XDefaultVisual(self.dpy, self.screen), XDefaultColormap(self.dpy, self.screen), clrname, dest) == 0 {
		Die::stderr(format!("error, cannot allocate color {:?}", CStr::from_ptr(clrname)))
	    } else {
		Ok(())
	    }
	}
    }

    fn fontset_create(&mut self) -> CompResult<()> {
	for font in self.config.fontstrings.iter_mut() {
	    font.push('\0');
	}
	for font in self.config.fontstrings.iter() {
	    self.fonts.push(Fnt::new(self, Some(font), ptr::null_mut())?);
	}

	Ok(())
    }
}
