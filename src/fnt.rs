use x11::xlib::{Display};
use x11::xft::{XftFontClose, FcPattern, XftFontOpenPattern, 
	       XftNameParse, XftFontOpenName, XftFont};
use fontconfig::fontconfig::{FcPatternDestroy, FcResultMatch, FcPatternGetBool, FcBool};
use crate::additional_bindings::fontconfig::FC_COLOR;
use std::ptr;
use std::ffi::{CStr, c_void};
use libc::{c_uint, c_char};
use std::mem::MaybeUninit;

use crate::drw::Drw;

#[derive(Debug)]
pub struct Fnt {
    pub xfont: *mut XftFont,
    pub pattern_pointer: *mut FcPattern,
    pub height: c_uint,
}

impl PartialEq for Fnt {
    fn eq(&self, other: &Self) -> bool {
	self.xfont == other.xfont
    }
}

impl Fnt {
    // xfont_create
    pub fn new(drw: &Drw, fontname: *mut c_char, mut pattern: *mut FcPattern) -> Option<Self> {
	unsafe {
	    let xfont;
	    if fontname != ptr::null_mut() {
		/* Using the pattern found at font->xfont->pattern does not yield the
		 * same substitution results as using the pattern returned by
		 * FcNameParse; using the latter results in the desired fallback
		 * behaviour whereas the former just results in missing-character
		 * rectangles being drawn, at least with some fonts. */
		xfont = XftFontOpenName(drw.dpy, drw.screen, fontname);
		if xfont == ptr::null_mut() {
		    eprintln!("error, cannot load font from name: '%s'\n");
		    return None;
		}
		pattern = XftNameParse(fontname);
		if pattern == ptr::null_mut() {
		    let c_str: &CStr = CStr::from_ptr(fontname);
		    let str_slice: &str = c_str.to_str().unwrap();
		    eprintln!("error, cannot parse font name to pattern: '{}'", str_slice);
		    XftFontClose(drw.dpy, xfont);
		    return None;
		}
	    } else if pattern != ptr::null_mut() {
		xfont = XftFontOpenPattern(drw.dpy, pattern);
		if xfont == ptr::null_mut() {
		    eprintln!("error, cannot load font from pattern.");
		    return None; // return to clean up
		}
	    } else {
		eprintln!("No font specified.");
		return None;
	    }

	    
	    /* Do not allow using color fonts. This is a workaround for a BadLength
	     * error from Xft with color glyphs. Modelled on the Xterm workaround. See
	     * https://bugzilla.redhat.com/show_bug.cgi?id=1498269
	     * https://lists.suckless.org/dev/1701/30932.html
	     * https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=916349
	     * and lots more all over the internet.
	     */

	    let mut iscol: FcBool = MaybeUninit::uninit().assume_init();
	    if FcPatternGetBool(pattern as *mut c_void, FC_COLOR, 0, &mut iscol) == FcResultMatch && iscol != 0 {
		XftFontClose(drw.dpy, xfont);
		return None;
	    }

	    let height = (*xfont).ascent+(*xfont).descent;

	    return Some(Self{xfont, pattern_pointer: pattern, height: height as c_uint});
	}
    }
    // xfont_free
    pub fn free(&mut self, dpy: *mut Display) {
	unsafe {
	    if self.pattern_pointer != ptr::null_mut() {
		FcPatternDestroy(self.pattern_pointer as *mut c_void);
	    }
	    XftFontClose(dpy, self.xfont);
	}
    }
}
