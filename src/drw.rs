use x11::xlib::{Display, Window, Drawable, GC,
		XWindowAttributes, XFreeGC,
		XUngrabKey,
		XDefaultColormap, XDefaultVisual, False, 
		XFillRectangle, XSetForeground, 
		AnyKey,
		XDrawRectangle, XCopyArea, 
		XSync, AnyModifier, XCloseDisplay,
		XFreePixmap};
use x11::xft::{XftColor, FcPattern, XftDrawStringUtf8,
	       XftDraw, XftDrawCreate,
	       XftTextExtentsUtf8, XftCharExists, XftFontMatch, XftDrawDestroy};
use x11::xrender::XGlyphInfo;
use fontconfig::fontconfig::{FcPatternAddBool, FcPatternDestroy,
			     FcCharSetCreate, FcCharSetAddChar, FcPatternDuplicate, FcPatternAddCharSet,
			     FcCharSetDestroy, FcDefaultSubstitute, FcMatchPattern, FcConfigSubstitute};
use crate::additional_bindings::fontconfig::{FC_SCALABLE, FC_CHARSET, FC_COLOR, FcTrue, FcFalse};
use libc::{c_uchar, c_int, c_uint, c_void, free};
use std::{mem::{MaybeUninit, ManuallyDrop}, ptr};

use crate::item::{Items, Direction::*};
use crate::globals::*;
use crate::config::{*, Schemes::*, Clrs::*};
use crate::fnt::*;

pub enum TextOption<'a> {
    Prompt,
    Input,
    Other(&'a String),
}
use TextOption::*;

#[derive(Debug)]
pub struct Drw {
    pub wa: XWindowAttributes,
    pub dpy: *mut Display,
    pub screen: c_int,
    pub root: Window,
    pub drawable: Drawable,
    pub gc: GC,
    pub scheme: [*mut XftColor; 2],
    pub fonts: Vec<Fnt>,
    pub pseudo_globals: PseudoGlobals,
    pub w: c_int,
    pub h: c_int,
    pub config: Config,
    pub input: String,
    pub items: ManuallyDrop<Items>,
}

impl Drw {
    pub fn fontset_getwidth(&mut self, text: TextOption) -> Result<c_int, String> {
	if self.fonts.len() == 0 {
	    Ok(0)
	} else {
	    self.text(0, 0, 0, 0, 0, text, false)
	}
    }

    pub fn text(&mut self, mut x: c_int, y: c_int, mut w: c_uint, h: c_uint, lpad: c_uint, text_opt: TextOption, invert: bool) -> Result<c_int, String> {
	let text = {
	    match text_opt {
		Prompt => &self.config.prompt,
		Input => &self.input,
		Other(string) => string,
	    }
	};
	unsafe {
	    
	    let render = x>0 || y>0 || w>0 || h>0;

	    if text.len() == 0 || self.fonts.len() == 0 {
		return Ok(0);
	    }
	    
	    let mut d: *mut XftDraw = ptr::null_mut();

	    if !render {
		w = !0; // maximize w so that underflow never occurs
	    } else {
		XSetForeground(self.dpy, self.gc, (*self.scheme[if invert {ColFg} else {ColBg} as usize]).pixel);
		XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w as u32, h);
		d = XftDrawCreate(self.dpy, self.drawable,
		                  XDefaultVisual(self.dpy, self.screen),
		                  XDefaultColormap(self.dpy, self.screen));
		x += lpad as c_int;
		w -= lpad;
	    }

	    let mut slice_start = 0;
	    let mut slice_end = 0;
	    let mut cur_font: Option<usize> = None;
	    
	    for cur_char in text.chars() {
		// String is already utf8 so we don't need to do extra conversions
		// As such, this logic is changed from the source dmenu quite a bit

		let mut found_font = self.fonts.iter().position(|font| XftCharExists(self.dpy, font.xfont, cur_char as u32) == 1);
		if cur_font == found_font {
		    // append to list to be printed
		    slice_end += cur_char.len_utf8();
		}
		if cur_font != found_font {
		    if found_font.is_none() {
			// char is not found in any fonts
			// In this case, pretend it's in the first font, as it must be drawn
			
			let fccharset = FcCharSetCreate();
			FcCharSetAddChar(fccharset, cur_char as u32);
			if self.fonts[0].pattern_pointer == ptr::null_mut() {
			    /* Refer to the comment in xfont_create for more information. */
			    return Err(format!("fonts must be loaded from font strings"));
			}
			
			let fcpattern = FcPatternDuplicate(self.fonts[0].pattern_pointer as *const c_void);
			FcPatternAddCharSet(fcpattern as *mut c_void, FC_CHARSET, fccharset);
			FcPatternAddBool(fcpattern as *mut c_void, FC_SCALABLE, FcTrue);
			FcPatternAddBool(fcpattern as *mut c_void, FC_COLOR, FcFalse);

			FcConfigSubstitute(ptr::null_mut(), fcpattern as *mut c_void, FcMatchPattern);
			FcDefaultSubstitute(fcpattern as *mut c_void);
			let mut result = MaybeUninit::uninit().assume_init(); // XftFontMatch isn't null safe so we need some memory
			let font_match = XftFontMatch(self.dpy, self.screen, fcpattern as *const FcPattern, &mut result);

			FcCharSetDestroy(fccharset);
			FcPatternDestroy(fcpattern);

			
			if font_match != ptr::null_mut() {
			    let usedfont_opt = Fnt::new(self, ptr::null_mut(), font_match);
			    if let Some(mut usedfont) = usedfont_opt {
				if XftCharExists(self.dpy, usedfont.xfont, cur_char as u32) != 0 {
				    found_font = Some(self.fonts.len());
				    self.fonts.push(usedfont);
				} else {
				    usedfont.free(self.dpy);
				    found_font = Some(0);
				}
			    } else {
				found_font = Some(0);
			    }
			}
		    }
		    // Need to switch fonts
		    // First, take care of the stuff pending print
		    self.render(&mut x, &y, &mut w, &h,
				text.as_ptr().offset(slice_start as isize), slice_end-slice_start,
				&cur_font, d, render, invert);
		    // Then, set up next thing to print
		    cur_font = found_font;
		    slice_start = slice_end;
		    slice_end += cur_char.len_utf8();
		}
	    }
	    // take care of the remaining slice, if it exists
	    self.render(&mut x, &y, &mut w, &h,
			text.as_ptr().offset(slice_start as isize), slice_end-slice_start,
			&cur_font, d, render, invert);
	    
	    if d != ptr::null_mut() {
		XftDrawDestroy(d);
	    }

	    Ok(x + if render {w} else {0} as i32)
	}
    }

    fn render(&self, x: &mut i32, y: &i32, w: &mut u32, h: &u32, subtext: *const c_uchar, len: usize, cur_font: &Option<usize>, d: *mut XftDraw, render: bool, invert: bool) {
	if len == 0 {
	    return;
	}
	unsafe {
	    let usedfont = cur_font.map(|i| &self.fonts[i]).unwrap();
	    let font_ref = usedfont;
	    let (substr_width, _) = self.font_getexts(font_ref, subtext, len as c_int);
	    if render {
		let ty = *y + (*h as i32 - usedfont.height as i32) / 2 + (*usedfont.xfont).ascent;
		XftDrawStringUtf8(d, self.scheme[if invert {ColBg} else {ColFg} as usize],  self.fonts[cur_font.unwrap()].xfont, *x, ty, subtext, len as c_int);
	    }
	    *x += substr_width as i32;
	    *w -= substr_width;
	}
    }

    pub fn font_getexts(&self, font: &Fnt, subtext: *const c_uchar, len: c_int) -> (c_uint, c_uint) {
	unsafe { //                                                                (width,  height)
	    let mut ext: XGlyphInfo = MaybeUninit::uninit().assume_init();
	    XftTextExtentsUtf8(self.dpy, font.xfont, subtext, len, &mut ext);
	    (ext.xOff as c_uint, font.height) // (width, height)
	}
    }

    pub fn draw(&mut self) -> Result<(), String> { // drawmenu
	self.setscheme(SchemeNorm);
	self.rect(0, 0, self.w as u32, self.h as u32, true, true); // clear menu

	let mut x = 0;
	
	if self.config.prompt.len() > 0 { // draw prompt
	    self.setscheme(SchemeSel);
	    match self.text(x, 0, self.pseudo_globals.promptw as c_uint,
			    self.pseudo_globals.bh as u32, self.pseudo_globals.lrpad as u32 / 2, Prompt, false) {
		Ok(computed_width) => x = computed_width,
		Err(err) => return Err(err),
	    }
	}
	
	/* draw input field */
	if let Err(err) = Items::gen_matches(self, if self.config.lines > 0 {Vertical} else {Horizontal}) {
	    return Err(err);
	}
	let w = if self.config.lines > 0 || self.items.match_len() == 0 {
	    self.w - x
	} else {
	    self.pseudo_globals.inputw
	};
	self.setscheme(SchemeNorm);
	if let Err(err) = self.text(x, 0, w as c_uint, self.pseudo_globals.bh as c_uint,
				    self.pseudo_globals.lrpad as c_uint / 2, Input, false) {
	    return Err(err);
	}
	match self.textw(Input) {
	    Ok(inputw) => {
		match self.textw(Other(&self.input[self.pseudo_globals.cursor..].to_string())) {
		    Ok(otherw) => {
			let curpos: c_int = inputw - otherw + self.pseudo_globals.lrpad/2 - 1;

			if curpos < w {
			    self.setscheme(SchemeNorm);
			    self.rect(x + curpos, 2, 2, self.pseudo_globals.bh as u32 - 4, true, false);
			}

			if let Err(err) = Items::draw(self, if self.config.lines > 0 {Vertical} else {Horizontal}) {
			    return Err(err);
			}

			self.map(self.pseudo_globals.win, 0, 0, self.w, self.h);
			Ok(())
		    },
		    Err(err) => return Err(err),
		}
	    },
	    Err(err) => return Err(err),
	}
    }

    pub fn map(&self, win: Window, x: c_int, y: c_int, w: c_int, h: c_int) {
	unsafe {
	    XCopyArea(self.dpy, self.drawable, win, self.gc, x, y, w as u32, h as u32, x, y);
	    XSync(self.dpy, False);
	}
    }

    pub fn textw(&mut self, text: TextOption) -> Result<c_int, String> {
	self.fontset_getwidth(text).map(|computed_width| computed_width + self.pseudo_globals.lrpad)
    }
    
    pub fn setscheme(&mut self, scm: Schemes) {
	self.scheme = self.pseudo_globals.schemeset[scm as usize];
    }

    fn rect(&self, x: c_int, y: c_int, w: c_uint, h: c_uint, filled: bool, invert: bool) {
	unsafe {
	    XSetForeground(self.dpy, self.gc, (*self.scheme[if invert {ColBg} else {ColFg} as usize]).pixel);
	    if filled {
		XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w, h);
	    } else {
		XDrawRectangle(self.dpy, self.drawable, self.gc, x, y, w - 1, h - 1);
	    }
	}
    }
}

impl Drop for Drw {
    fn drop(&mut self) {
	unsafe {
	    for font in &mut self.fonts {
		font.free(self.dpy);
	    }
	    ManuallyDrop::drop(&mut self.items);
	    XUngrabKey(self.dpy, AnyKey, AnyModifier, self.root);
	    for i in 0..SchemeLast as usize{
		free(self.pseudo_globals.schemeset[i][0] as *mut c_void);
		free(self.pseudo_globals.schemeset[i][1] as *mut c_void);
	    }
	    XFreePixmap(self.dpy, self.drawable);
	    XFreeGC(self.dpy, self.gc);
	    XSync(self.dpy, False);
	    XCloseDisplay(self.dpy);
	}
    }
}
