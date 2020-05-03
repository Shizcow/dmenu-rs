use x11::xlib::{Display, Window, Drawable, GC, XCreateGC, XCreatePixmap, XSetLineAttributes,
		XDefaultDepth, XWindowAttributes, JoinMiter, CapButt, LineSolid, XGetWindowAttributes,
		XDefaultColormap, XDefaultVisual, XClassHint, True, False, XInternAtom, Atom,
		XFillRectangle, XSetForeground};
use x11::xft::{XftFont, XftColor, FcPattern, XftFontOpenPattern, XftFontOpenName,
	       XftFontClose, XftNameParse, XftColorAllocName, XftDraw, XftDrawCreate,
	       XftTextExtentsUtf8, XftCharExists};
use x11::xrender::{XRenderColor, XGlyphInfo};
use fontconfig::fontconfig::{FcResultMatch, FcPatternGetBool, FcBool, FcPatternAddBool,
			     FcCharSetCreate, FcCharSetAddChar, FcPatternDuplicate, FcPatternAddCharSet};
use crate::additional_bindings::fontconfig::{FC_SCALABLE, FC_CHARSET, FC_COLOR, FcTrue, FcFalse};
use std::ptr;
use std::ffi::{CString, CStr, c_void};
use libc::{c_char, c_uchar, c_int, c_uint};

use std::mem::{self, MaybeUninit};

use crate::config::{COLORS, Schemes, Clrs, Config};

type Clr = XftColor;

#[derive(Debug)]
pub struct PseudoGlobals {
    promptw: c_int,
    lrpad: c_int,
    schemeset: [*mut Clr; Schemes::SchemeLast as usize], // replacement for "scheme"
}

impl Default for PseudoGlobals {
    fn default() -> Self {
	unsafe {
	    Self {
		promptw:   MaybeUninit::uninit().assume_init(),
		schemeset: MaybeUninit::uninit().assume_init(),
		lrpad:     MaybeUninit::uninit().assume_init(),
	    }
	}
    }
}


#[derive(Debug)]
struct Fnt {
    xfont: *mut XftFont,
    pattern_pointer: *mut FcPattern,
    height: c_uint,
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
		    return None;
		}
	    } else {
		panic!("No font specified.");
	    }

	    /* Do not allow using color fonts. This is a workaround for a BadLength
	     * error from Xft with color glyphs. Modelled on the Xterm workaround. See
	     * https://bugzilla.redhat.com/show_bug.cgi?id=1498269
	     * https://lists.suckless.org/dev/1701/30932.html
	     * https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=916349
	     * and lots more all over the internet.
	     */

	    let iscol = MaybeUninit::uninit().assume_init();
	    let fc_color = MaybeUninit::uninit().assume_init();
	    let mut pattern_pointer = pattern as *mut c_void;
	    if(FcPatternGetBool(pattern_pointer, fc_color, 0, iscol) == FcResultMatch && *iscol != 0) {
		XftFontClose(drw.dpy, xfont);
		return None;
	    }

	    let height = (*xfont).ascent+(*xfont).descent;

	    return Some(Self{xfont, pattern_pointer: pattern_pointer as *mut FcPattern, height: height as c_uint});
	}
    }
}

#[derive(Debug)]
pub struct Drw {
    wa: XWindowAttributes,
    dpy: *mut Display,
    screen: c_int,
    root: Window,
    drawable: Drawable,
    gc: GC,
    scheme: Clr,
    fonts: Vec<Fnt>,
    pseudo_globals: PseudoGlobals,
}

impl Drw {
    pub fn new(dpy: *mut Display, screen: c_int, root: Window, wa: XWindowAttributes, mut pseudo_globals: PseudoGlobals) -> Self {
	unsafe {
	    let drawable = XCreatePixmap(dpy, root, wa.width as u32, wa.height as u32, XDefaultDepth(dpy, screen) as u32);
	    let gc = XCreateGC(dpy, root, 0, ptr::null_mut());
	    XSetLineAttributes(dpy, gc, 1, LineSolid, CapButt, JoinMiter);
	    let fonts = Vec::new();
	    let mut ret = Self{wa, dpy, screen, root, drawable, gc, fonts: fonts, pseudo_globals,
			       scheme: MaybeUninit::uninit().assume_init()};

	    for j in 0..(Schemes::SchemeLast as usize) {
		ret.pseudo_globals.schemeset[j] = ret.scm_create(COLORS[j]);
	    }
	    
	    ret
	}
    }

    pub fn fontset_create(&mut self, fonts: Vec<*mut c_char>) -> bool {
	if fonts.len() == 0 {
	    return false;
	}

	for font in fonts.into_iter().rev() {
	    let to_push = Fnt::new(self, font, ptr::null_mut());
	    if to_push.is_some() {
		self.fonts.push(to_push.unwrap());
	    }
	}

	true
    }

    fn scm_create(&self, clrnames: [[u8; 8]; 2]) -> *mut Clr {
	/* need at least two colors for a scheme */
	if clrnames.len() < 2 {
	    return ptr::null_mut();
	}
	
	let ret: *mut Clr = unsafe{ MaybeUninit::uninit().assume_init() };
	for clrname in clrnames.iter() {
	    self.clr_create(ret, clrname.as_ptr() as *const c_char);
	}
	ret
    }

    fn clr_create(&self, dest: *mut Clr, clrname: *const c_char) {
	unsafe {
	    if clrname == ptr::null_mut() {
		return;
	    }
	    if (XftColorAllocName(self.dpy, XDefaultVisual(self.dpy, self.screen), XDefaultColormap(self.dpy, self.screen), clrname, dest)==0) {
		panic!("error, cannot allocate color {:?}", CStr::from_ptr(clrname));
	    }
	}
    }

    pub fn setup(&mut self, config: Config, parentwin: u64) {
	let x: c_int;
	let y: c_int;
	let i: c_int;
	let j: c_int;
	
	let ch: XClassHint = XClassHint{
	    res_name: (*b"dmenu\0").as_ptr() as *mut c_char,
	    res_class: (*b"dmenu\0").as_ptr() as *mut c_char
	};

	// appearances are set up in constructor
	
	let clip: Atom = unsafe{ XInternAtom(self.dpy, (*b"CLIPBOARD\0").as_ptr()   as *mut c_char, False) };
	let utf8: Atom = unsafe{ XInternAtom(self.dpy, (*b"UTF8_STRING\0").as_ptr() as *mut c_char, False) };

	let bh: c_uint = self.fonts[0].height+2;
	// config.lines = config.lines.max(0); // Why is this in the source if lines is unsigned?
	let mh: c_uint = (config.lines)*bh;

	// TODO: XINERAMA

	{
	    if (unsafe{XGetWindowAttributes(self.dpy, parentwin, &mut self.wa)} == 0) {
		panic!("could not get embedding window attributes: 0x{:?}", parentwin);
	    }
	    x = 0;
	    y = if config.topbar != 0 {
		0
	    } else {
		self.wa.height - mh as c_int
	    };
	}
	
	self.pseudo_globals.promptw = if config.prompt.len() != 0 {
	    self.fontset_getwidth(&config.prompt) + (3/4)*self.pseudo_globals.lrpad
	} else {
	    0
	};
	
    }

    fn fontset_getwidth(&self, text: &String) -> c_int {
	if(self.fonts.len() == 0) {
	    0
	} else {
	    self.text(0, 0, 0, 0, 0, text, false)
	}
    }

    fn text(&self, mut x: c_int, y: c_int, mut w: c_uint, h: c_uint, lpad: c_uint, text: &String, invert: bool) -> c_int { // TODO: can invert be a bool?
	unsafe {
	    /*
	    let buf: [c_uchar; 1024];
	    let ty: c_int;
	    let ew: c_uint;
	    let usedfont: *mut Fnt = MaybeUninit::uninit().assume_init();
	    let curfont:  *mut Fnt = MaybeUninit::uninit().assume_init();
	    let nextfont: *mut Fnt = MaybeUninit::uninit().assume_init();
	    let i: usize = MaybeUninit::uninit().assume_init();
	    let len: usize = MaybeUninit::uninit().assume_init();
	     */
	    
	    let render = x>0 || y>0 || w>0 || h>0;

	    if text.len() == 0 || self.fonts.len() == 0 { //self.scheme isn't statically initalized null check here is useless
		return 0;
	    }
	    
	    let mut d: *mut XftDraw = ptr::null_mut();

	    if !render {
		w = !w; // bitwise not
	    } else {
		XSetForeground(self.dpy, self.gc, (*self.pseudo_globals.schemeset[if invert {Clrs::ColFg} else {Clrs::ColBg} as usize]).pixel);
		XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w, h);
		d = XftDrawCreate(self.dpy, self.drawable,
		                  XDefaultVisual(self.dpy, self.screen),
		                  XDefaultColormap(self.dpy, self.screen));
		x += lpad as c_int;
		w -= lpad;
	    }
	    
	    //let usedfont = &self.fonts[0];

	    let mut slice_start = 0;
	    let mut slice_end = 0;
	    let mut cur_font: Option<&Fnt> = None;
	    
	    for cur_char in text.chars() {
		// String is already utf8 so we don't need to do extra conversions
		// As such, this logic is changed from the source dmenu quite a bit

		let found_font = self.fonts.iter().find(|font| XftCharExists(self.dpy, font.xfont, cur_char as u32) == 1);
		if cur_font == found_font {
		    // append to list to be printed
		    slice_end += cur_char.len_utf8();
		} else {
		    if found_font.is_none() {
			// char is not found in any fonts
			// In this case, pretend it's in the first font, as it must be drawn
			let fccharset = FcCharSetCreate();
			FcCharSetAddChar(fccharset, cur_char as u32);
			if (cur_font.unwrap().pattern_pointer == ptr::null_mut()) {
				/* Refer to the comment in xfont_create for more information. */
				panic!("fonts must be loaded from font strings");
			}
			let fcpattern = FcPatternDuplicate(cur_font.unwrap().pattern_pointer as *const c_void);
			FcPatternAddCharSet(fcpattern, FC_CHARSET, fccharset);
			FcPatternAddBool(fcpattern, FC_SCALABLE, FcTrue);
			FcPatternAddBool(fcpattern, FC_COLOR, FcFalse);

			// NOT DONE YET

			// Now, check if we need to render it or if we can wait
			if cur_font == Some(&self.fonts[0]) {
			    slice_end += cur_char.len_utf8();
			    continue;
			} else {
			    cur_font = Some(&self.fonts[0]);
			}
		    }
		    // Need to switch fonts
		    // First, take care of the stuff pending print
		    if(slice_start != slice_end){
			println!("Ok to print: '{}' with a length of {}", std::str::from_utf8(&text.as_bytes()[slice_start..slice_end]).unwrap(), slice_end-slice_start);
		    }
		    // Then, set up next thing to print
		    cur_font = found_font;
		    slice_start = slice_end;
		    slice_end += cur_char.len_utf8();
		}
	    }
	    // take care of the remaining slice, if it exists
	    if(slice_start != slice_end){
		println!("Ok to print: '{}' with a length of {}", std::str::from_utf8(&text.as_bytes()[slice_start..slice_end]).unwrap(), slice_end-slice_start);
	    }

	    0
	}
    }

    fn font_getexts(&self, font: &Fnt, subtext: *const c_uchar, len: c_int) -> (c_uint, c_uint) { // (width, height)
	if (len == 0) { // font == ptr::null() is always false
	    return (0, 0); // TODO: is this actually required?
	}
	
	let mut ext: XGlyphInfo = unsafe{MaybeUninit::uninit().assume_init()};

	unsafe{XftTextExtentsUtf8(self.dpy, font.xfont, subtext, len, &mut ext)};

	(ext.xOff as c_uint, font.height) // (width, height)
    }
}
