use x11::xlib::Display;
use x11::xft::{XftFontClose, FcPattern, XftFontOpenPattern, 
	       XftNameParse, XftFontOpenName, XftFont};
use fontconfig::fontconfig::{FcPatternDestroy, FcResultMatch, FcPatternGetBool, FcBool, 
			     FcFontList, FcObjectSetDestroy, FcFontSetDestroy,
			     FcChar8, FcObjectSetBuild, FcNameParse,
			     FcFontSet};
use crate::additional_bindings::fontconfig::{FC_COLOR, FC_FAMILY};
use std::ptr;
use std::ffi::c_void;
use libc::c_uint;
use std::mem::MaybeUninit;

use crate::drw::Drw;
use crate::result::*;

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
    pub fn new(drw: &Drw, fontopt: Option<&String>, mut pattern: *mut FcPattern) -> CompResult<Self> {
	let __blank = "".to_owned(); // fighting the borrow checker
	let fontname = fontopt.unwrap_or(&__blank);
	let fontptr = if fontname.len() > 0 {
	    fontname.as_ptr() as *mut i8
	} else {
	    ptr::null_mut()
	};
	unsafe {
	    let xfont;
	    if fontptr != ptr::null_mut() {

		if let Err(warning) = Self::find_font_sys(&fontname) {
		    eprintln!("{}", warning);
		}
		
		/* Using the pattern found at font->xfont->pattern does not yield the
		 * same substitution results as using the pattern returned by
		 * FcNameParse; using the latter results in the desired fallback
		 * behaviour whereas the former just results in missing-character
		 * rectangles being drawn, at least with some fonts. */
		xfont = XftFontOpenName(drw.dpy, drw.screen, fontptr);
		if xfont == ptr::null_mut() {
		    return Die::stderr(format!("error, cannot load font from name: '{}'", fontname));
		}
		
		pattern = XftNameParse(fontptr);
		if pattern == ptr::null_mut() {
		    XftFontClose(drw.dpy, xfont);
		    return Die::stderr(format!("error, cannot parse font name to pattern: '{}'",
				       fontname));
		}
	    } else if pattern != ptr::null_mut() {
		xfont = XftFontOpenPattern(drw.dpy, pattern);
		if xfont == ptr::null_mut() {
		    return Die::stderr(format!("error, cannot load font '{}' from pattern.",
				       fontname));
		}
	    } else {
		return Die::stderr("No font specified.".to_owned());
	    }

	    
	    /* Do not allow using color fonts. This is a workaround for a BadLength
	     * error from Xft with color glyphs. Modelled on the Xterm workaround. See
	     * https://bugzilla.redhat.com/show_bug.cgi?id=1498269
	     * https://lists.suckless.org/dev/1701/30932.html
	     * https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=916349
	     * and lots more all over the internet.
	     */
	    let mut iscol: FcBool = MaybeUninit::uninit().assume_init();
	    if FcPatternGetBool((*xfont).pattern as *mut c_void, FC_COLOR, 0, &mut iscol) == FcResultMatch
		&& iscol != 0 {
		XftFontClose(drw.dpy, xfont);
		return Die::stderr("Cannot load color fonts".to_owned());
	    }

	    let height = (*xfont).ascent+(*xfont).descent;

	    return Ok(Self{xfont, pattern_pointer: pattern, height: height as c_uint});
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

    fn find_font_sys(needle: &String) -> Result<(), String> {
	// First, validate the font name
	let parts = needle.split(":").nth(0).unwrap_or("")
	    .split("-").collect::<Vec<&str>>();
	let searchterm = {
	    let len = parts.len();
	    parts.into_iter().take(if len <= 1 {len} else {len-1})
		.fold(String::new(), |mut acc, g| {acc.push_str(g); acc})
	};

	// Then search for it on the system
	unsafe {
	    let all_fonts = Self::op_pattern("".to_owned())?;
	    let mut fs = Self::op_pattern(searchterm.clone())?;
	    if (*fs).nfont == 0 { // user may have searched for an attribute instead of family
		FcFontSetDestroy(fs);
		fs = Self::op_pattern(format!(":{}", searchterm))?;
	    }
	    let ret = 
		if (*fs).nfont == 0 || (*fs).nfont == (*all_fonts).nfont {
		    Err(format!("Warning: font '{}' not found on the system", searchterm))
		} else {
		    Ok(())
		};
	    FcFontSetDestroy(fs);
	    FcFontSetDestroy(all_fonts);
	    ret
	}
    }

    fn op_pattern(mut pattern: String) -> Result<*mut FcFontSet, String> {
	unsafe {
	    pattern.push('\0');
	    let os = FcObjectSetBuild(FC_FAMILY, ptr::null_mut::<c_void>());
	    let pat = FcNameParse(format!("{}\0", pattern).as_ptr() as *mut FcChar8);
	    let fs = FcFontList(ptr::null_mut(), pat, os);
	    FcPatternDestroy(pat);
	    FcObjectSetDestroy(os);
	    if fs == ptr::null_mut() {
		return Err("Could not get system fonts".to_owned());
	    }
	    Ok(fs)
	}
    }
}
