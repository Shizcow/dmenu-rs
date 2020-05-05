use x11::xlib::{Display, Window, Drawable, GC, XCreateGC, XCreatePixmap, XSetLineAttributes,
		XDefaultDepth, XWindowAttributes, JoinMiter, CapButt, LineSolid, XGetWindowAttributes,
		XDefaultColormap, XDefaultVisual, XClassHint, True, False, XInternAtom, Atom,
		XFillRectangle, XSetForeground, XSetClassHint, CWEventMask, CWBackPixel,
		CWOverrideRedirect, XCreateWindow, VisibilityChangeMask, KeyPressMask, ExposureMask,
		XSetWindowAttributes, CopyFromParent, Visual, XOpenIM, XNFocusWindow, XNClientWindow,
		XIMStatusNothing, XIMPreeditNothing, XNInputStyle, XCreateIC};
use x11::xft::{XftFont, XftColor, FcPattern, XftFontOpenPattern, XftFontOpenName, XftDrawStringUtf8,
	       XftFontClose, XftNameParse, XftColorAllocName, XftDraw, XftDrawCreate,
	       XftTextExtentsUtf8, XftCharExists, XftFontMatch, XftDrawDestroy};
use x11::xrender::{XRenderColor, XGlyphInfo};
use fontconfig::fontconfig::{FcResultMatch, FcPatternGetBool, FcBool, FcPatternAddBool, FcPatternDestroy,
			     FcCharSetCreate, FcCharSetAddChar, FcPatternDuplicate, FcPatternAddCharSet,
			     FcCharSetDestroy, FcDefaultSubstitute, FcMatchPattern, FcConfigSubstitute};
use crate::additional_bindings::fontconfig::{FC_SCALABLE, FC_CHARSET, FC_COLOR, FcTrue, FcFalse};
#[cfg(feature = "Xinerama")]
use x11::xinerama::{XineramaQueryScreens, XineramaScreenInfo};
#[cfg(feature = "Xinerama")]
use x11::xlib::{XGetInputFocus, PointerRoot, XFree, XQueryTree, XQueryPointer};
use std::ptr;
use std::ffi::{CString, CStr, c_void};
use libc::{c_char, c_uchar, c_int, c_uint, c_short, exit};

use std::mem::{self, MaybeUninit};

use crate::config::{COLORS, Schemes, Clrs, Config};
use crate::item::Item;

type Clr = XftColor;

#[cfg(feature = "Xinerama")]
fn intersect(x: c_int, y: c_int, w: c_int, h: c_int, r: *mut XineramaScreenInfo) -> c_int {
    unsafe {
	0.max((x+w).min(((*r).x_org+(*r).width) as c_int) - x.max((*r).x_org as c_int)) *
	    0.max((y+h).min(((*r).y_org+(*r).height) as c_int) - y.max((*r).y_org as c_int))
    }
}

#[derive(Debug)]
pub struct PseudoGlobals {
    pub promptw: c_int,
    pub lrpad: c_int,
    pub schemeset: [*mut Clr; Schemes::SchemeLast as usize], // replacement for "scheme"
    pub mon: c_int,
    pub mw: c_int,
    pub mh: c_int,
    pub win: Window,
}

impl Default for PseudoGlobals {
    fn default() -> Self {
	unsafe {
	    Self {
		promptw:   MaybeUninit::uninit().assume_init(),
		schemeset: MaybeUninit::uninit().assume_init(),
		lrpad:     MaybeUninit::uninit().assume_init(),
		mon:       -1,
		mw:         MaybeUninit::uninit().assume_init(),
		mh:         MaybeUninit::uninit().assume_init(),
		win:        MaybeUninit::uninit().assume_init(),
	    }
	}
    }
}


#[derive(Debug)]
pub struct Fnt {
    xfont: *mut XftFont,
    pattern_pointer: *mut FcPattern,
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

	    let mut iscol: FcBool = MaybeUninit::uninit().assume_init();
	    let mut pattern_pointer = pattern as *mut c_void;
	    if(FcPatternGetBool(pattern_pointer, FC_COLOR, 0, &mut iscol) == FcResultMatch && iscol != 0) {
		XftFontClose(drw.dpy, xfont);
		return None;
	    }

	    let height = (*xfont).ascent+(*xfont).descent;

	    return Some(Self{xfont, pattern_pointer: pattern_pointer as *mut FcPattern, height: height as c_uint});
	}
    }
    // xfont_free
    pub fn free(&mut self, dpy: *mut Display) { // TODO: impl Drop (with dpy param somehow)
	unsafe {
	    if(self.pattern_pointer != ptr::null_mut()) {
		FcPatternDestroy(self.pattern_pointer as *mut c_void);
	    }
	    XftFontClose(dpy, self.xfont);
	}
    }
}

#[derive(Debug)]
pub struct Drw {
    wa: XWindowAttributes,
    pub dpy: *mut Display,
    screen: c_int,
    root: Window,
    drawable: Drawable,
    gc: GC,
    schemes: [[Clr; 2]; Schemes::SchemeLast as usize], // TODO: vec or array?
    pub fonts: Vec<Fnt>,
    pub pseudo_globals: PseudoGlobals,
}

impl Drw {
    pub fn new(dpy: *mut Display, screen: c_int, root: Window, wa: XWindowAttributes, mut pseudo_globals: PseudoGlobals) -> Self {
	unsafe {
	    let drawable = XCreatePixmap(dpy, root, wa.width as u32, wa.height as u32, XDefaultDepth(dpy, screen) as u32);
	    let gc = XCreateGC(dpy, root, 0, ptr::null_mut());
	    XSetLineAttributes(dpy, gc, 1, LineSolid, CapButt, JoinMiter);
	    let fonts = Vec::new();
	    let mut ret = Self{wa, dpy, screen, root, drawable, gc, fonts: fonts, pseudo_globals,
			       schemes: MaybeUninit::uninit().assume_init()};

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
	
	let mut ret: Clr = unsafe{Clr{pixel: MaybeUninit::uninit().assume_init(), color: MaybeUninit::uninit().assume_init()}}; // need to alloc memmory
	for clrname in clrnames.iter() {
	    self.clr_create(&mut ret, clrname.as_ptr() as *const c_char);
	}
	&mut ret
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

    pub fn setup(&mut self, mut config: Config, parentwin: u64, root: u64, items: Vec<Item>) {
	unsafe {
	    let mut x: c_int = MaybeUninit::uninit().assume_init();
	    let mut y: c_int = MaybeUninit::uninit().assume_init();
	    let mut i: c_int = MaybeUninit::uninit().assume_init();
	    let mut j: c_int = MaybeUninit::uninit().assume_init();
	    
	    let mut ch: XClassHint = XClassHint{
		res_name: (*b"dmenu\0").as_ptr() as *mut c_char,
		res_class: (*b"dmenu\0").as_ptr() as *mut c_char
	    };

	    // appearances are set up in constructor
	    
	    let clip: Atom = unsafe{ XInternAtom(self.dpy, (*b"CLIPBOARD\0").as_ptr()   as *mut c_char, False) };
	    let utf8: Atom = unsafe{ XInternAtom(self.dpy, (*b"UTF8_STRING\0").as_ptr() as *mut c_char, False) };

	    let bh: c_uint = self.fonts[0].height+2;
	    // config.lines = config.lines.max(0); // Why is this in the source if lines is unsigned?
	    let mh: c_uint = (config.lines)*bh;

	    
	    if cfg!(feature = "Xinerama") {
		let mut i = 0;
		let mut area = 0;
		let mut n:  c_int  = MaybeUninit::uninit().assume_init();
		let mut di: c_int  = MaybeUninit::uninit().assume_init();
		let mut a:  c_int  = MaybeUninit::uninit().assume_init();
		let mut du: c_uint = MaybeUninit::uninit().assume_init();
		let mut w:  Window = MaybeUninit::uninit().assume_init();
		let mut dw: Window = MaybeUninit::uninit().assume_init();
		let mut pw: Window = MaybeUninit::uninit().assume_init();
		let mut dws: *mut Window = MaybeUninit::uninit().assume_init();
		let mut info = MaybeUninit::uninit().assume_init();
		if (parentwin == root) {
		    info = XineramaQueryScreens(self.dpy, &mut n);
		    if info != ptr::null_mut() {
			XGetInputFocus(self.dpy, &mut w, &mut di);
		    }
		    if self.pseudo_globals.mon >= 0 && self.pseudo_globals.mon < n {
			i = self.pseudo_globals.mon;
		    } else if w != root && w != PointerRoot as u64 && w != 0 {
			/* find top-level window containing current input focus */
			while {
			    pw = w;
			    if XQueryTree(self.dpy, pw, &mut dw, &mut w, &mut dws, &mut du) != 0 && dws != ptr::null_mut() {
				XFree(dws as *mut c_void);
			    }
			    (w != root && w != pw)
			} {} // do-while
			/* find xinerama screen with which the window intersects most */
			if (XGetWindowAttributes(self.dpy, pw, &mut self.wa) != 0) {
			    for j in 0..n {
				a = intersect(self.wa.x, self.wa.y, self.wa.width, self.wa.height, info.offset(j as isize));
				if a > area {
				    area = a;
				    i = j;
				}
			    }
			}
		    }
		}
		/* no focused window is on screen, so use pointer location instead */
		if (self.pseudo_globals.mon < 0 && area == 0 && XQueryPointer(self.dpy, root, &mut dw, &mut dw, &mut x, &mut y, &mut di, &mut di, &mut du) != 0) {
		    for i in 0..n {
			if (intersect(x, y, 1, 1, info.offset(i as isize)) != 0) {
			    break;
			}
		    }
		}
		x = (*info.offset(i as isize)).x_org as c_int;
		y = (*info.offset(i as isize)).y_org as c_int + (if config.topbar != 0 {0} else {(*info.offset(i as isize)).height as c_int - mh as c_int});
		self.pseudo_globals.mw = (*info.offset(i as isize)).width as c_int;
		XFree(info as *mut c_void);
	    } else {
		if (unsafe{XGetWindowAttributes(self.dpy, parentwin, &mut self.wa)} == 0) {
		    panic!("could not get embedding window attributes: 0x{:?}", parentwin);
		}
		x = 0;
		y = if config.topbar != 0 {
		    0
		} else {
		    self.wa.height - mh as c_int
		};
		self.pseudo_globals.mw = self.wa.width;
	    }
	    
	    self.pseudo_globals.promptw = if config.prompt.len() != 0 {
		self.fontset_getwidth(&config.prompt) + (3/4)*self.pseudo_globals.lrpad //TEXTW
	    } else {
		0
	    };
	    config.inputw = config.inputw.min(self.pseudo_globals.mw/3);

	    let mut swa: XSetWindowAttributes = MaybeUninit::uninit().assume_init();
	    swa.override_redirect = true as i32;
	    swa.background_pixel = self.schemes[Schemes::SchemeNorm as usize][Clrs::ColBg as usize].pixel;
	    swa.event_mask = ExposureMask | KeyPressMask | VisibilityChangeMask;
	    self.pseudo_globals.win =
		XCreateWindow(self.dpy, parentwin, x, y, self.pseudo_globals.mw as u32,
			      self.pseudo_globals.mh as u32, 0, CopyFromParent,
			      CopyFromParent as c_uint, CopyFromParent as *mut Visual,
			      CWOverrideRedirect | CWBackPixel | CWEventMask, &mut swa);
	    XSetClassHint(self.dpy, self.pseudo_globals.win, &mut ch);

	    /* input methods */
	    let xim = XOpenIM(self.dpy, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
	    if (xim == ptr::null_mut()) {
		panic!("XOpenIM failed: could not open input device");
	    }

	    // the following line segfaults
	    let xic = XCreateIC(xim, XNInputStyle, XIMPreeditNothing | XIMStatusNothing,
				XNClientWindow, self.pseudo_globals.win, XNFocusWindow,
				self.pseudo_globals.win, 0);
	    
	    panic!("Not done setting up");

	}
    }

    pub fn fontset_getwidth(&mut self, text: &String) -> c_int {
	if(self.fonts.len() == 0) {
	    0
	} else {
	    self.text(0, 0, 0, 0, 0, text, false)
	}
    }

    fn text(&mut self, mut x: c_int, y: c_int, mut w: c_uint, h: c_uint, lpad: c_uint, text: &String, invert: bool) -> c_int { // TODO: can invert be a bool?
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
	    let mut cur_font: Option<usize> = None;
	    
	    for (char_index, cur_char) in text.char_indices() {
		// String is already utf8 so we don't need to do extra conversions
		// As such, this logic is changed from the source dmenu quite a bit

		let mut found_font = self.fonts.iter().position(|font| XftCharExists(self.dpy, font.xfont, cur_char as u32) == 1);
		if cur_font == found_font {
		    // append to list to be printed
		    slice_end += cur_char.len_utf8();
		} else {
		    if found_font.is_none() {
			// char is not found in any fonts
			// In this case, pretend it's in the first font, as it must be drawn
			
			let fccharset = FcCharSetCreate();
			FcCharSetAddChar(fccharset, cur_char as u32);
			if (self.fonts[0].pattern_pointer == ptr::null_mut()) {
				/* Refer to the comment in xfont_create for more information. */
				panic!("fonts must be loaded from font strings");
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

			
			if (font_match != ptr::null_mut()) {
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
			

			// Now, check if we need to render it or if we can wait, TODO: impliment this as an optimization
			/*
			if cur_font == Some(0) {
			    slice_end += cur_char.len_utf8();
			    continue;
			} else {
			    cur_font = Some(0);
			}*/
		    }
		    // Need to switch fonts
		    // First, take care of the stuff pending print
		    if(slice_start != slice_end){
			let usedfont = cur_font.map(|i| &self.fonts[i]).unwrap();
			let font_ref = usedfont;
			let (substr_width, substr_height) = self.font_getexts(font_ref, text.as_ptr().offset(slice_start as isize), (slice_end-slice_start) as c_int);
			if render {
			    let ty = y + (h as i32 - usedfont.height as i32) / 2 + (*usedfont.xfont).ascent;
			    XftDrawStringUtf8(d, &self.schemes[0][if invert {Clrs::ColBg} else {Clrs::ColFg} as usize],  self.fonts[cur_font.unwrap()].xfont, x, ty, text.as_ptr().offset(slice_start as isize), (slice_end-slice_start) as c_int);
			}
			x += substr_width as i32;
			w -= substr_width;
		    }
		    // Then, set up next thing to print
		    cur_font = found_font;
		    slice_start = slice_end;
		    slice_end += cur_char.len_utf8();
		}
	    }
	    // take care of the remaining slice, if it exists
	    if(slice_start != slice_end){ // TODO: write once
		let usedfont = cur_font.map(|i| &self.fonts[i]).unwrap();
		let font_ref = usedfont;
		let (substr_width, substr_height) = self.font_getexts(font_ref, text.as_ptr().offset(slice_start as isize), (slice_end-slice_start) as c_int);
		if render {
		    let ty = y + (h as i32 - usedfont.height as i32) / 2 + (*usedfont.xfont).ascent;
		    XftDrawStringUtf8(d, &self.schemes[0][if invert {Clrs::ColBg} else {Clrs::ColFg} as usize],  self.fonts[cur_font.unwrap()].xfont, x, ty, text.as_ptr().offset(slice_start as isize), (slice_end-slice_start) as c_int);
		}
		x += substr_width as i32;
		w -= substr_width;
	    }
	    
	    if d != ptr::null_mut() {
		XftDrawDestroy(d);
	    }
	    
	    return x + if render {w} else {0} as i32; // TODO: make everything i32

	}
    }

    pub fn font_getexts(&self, font: &Fnt, subtext: *const c_uchar, len: c_int) -> (c_uint, c_uint) { // (width, height)
	if (len == 0) { // font == ptr::null() is always false
	    return (0, 0); // TODO: is this actually required?
	}
	
	let mut ext: XGlyphInfo = unsafe{MaybeUninit::uninit().assume_init()};

	unsafe{XftTextExtentsUtf8(self.dpy, font.xfont, subtext, len, &mut ext)};

	(ext.xOff as c_uint, font.height) // (width, height)
    }
}
