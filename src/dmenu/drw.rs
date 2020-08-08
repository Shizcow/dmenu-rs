//use crate::item::{Items, Direction::*};
//use crate::globals::*;
//use crate::config::*;
//use crate::fnt::*;
use crate::result::*;
use crate::init::*;

#[derive(PartialEq, Debug)]
pub enum TextOption<'a> {
    Prompt,
    Input,
    Other(&'a String),
}
use TextOption::*;

pub struct Drw {
    xkb_state: xkbcommon::xkb::State,
    conn: xcb::Connection,
    cr: cairo::Context,
    layout: pango::Layout,
}

// TODO: automate
const FONT:  &str = "Terminus 35";
const HEIGHT: u16 = 180;
const WIDTH:  u16 = 500;

impl Drw {
    pub fn new(/*pseudo_globals: PseudoGlobals, config: Config*/) -> CompResult<Self> {
	let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
	let (screen, window) = create_xcb_window(&conn, screen_num, 0, 0, WIDTH, HEIGHT);
	let xkb_state = setup_xkb(&conn, window);
	let cr = create_cairo_context(&conn, &screen, &window, WIDTH as i32, HEIGHT as i32);
	let layout = create_pango_layout(&cr, FONT);
	
	/*ret.items = if ret.config.nostdin {
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

	ret.config.lines = ret.config.lines.min(ret.get_items().len() as u32);
	 */

	Ok(Self{xkb_state, conn, layout, cr})
    }
}
/*
impl Drw {
    pub fn fontset_getwidth(&mut self, text: TextOption) -> CompResult<c_int> {
	if self.fonts.len() == 0 {
	    Ok(0)
	} else {
	    self.text(0, 0, 0, 0, 0, text, false).map(|o| o.0)
	}
    }

    pub fn text(&mut self, mut x: c_int, y: c_int, mut w: c_uint, h: c_uint, lpad: c_uint, text_opt: TextOption, invert: bool) -> CompResult<(c_int, Option<i32>)> {
	let mut text: String = {
	    match text_opt {
		Prompt => self.config.prompt.clone(),
		Input => self.format_input()?,
		Other(string) => string.to_string(),
	    }
	};
	unsafe {
	    
	    let render = x>0 || y>0 || w>0 || h>0;

	    if text.len() == 0 || self.fonts.len() == 0 {
		return Ok((0, None));
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
	    let mut spool = Spool::new();

	    text.push_str("."); // this will be removed later; turned into elipses
	    
	    for cur_char in text.chars() {
		// String is already utf8 so we don't need to do extra conversions
		// As such, this logic is changed from the source dmenu quite a bit

		let mut found_font = self.fonts.iter().position(|font| XftCharExists(self.dpy, font.xfont, cur_char as u32) == 1);
		if cur_font.is_some() && cur_font == found_font {
		    // append to list to be printed
		    slice_end += cur_char.len_utf8();
		}
		if cur_font.is_none() || cur_font != found_font {
		    if found_font.is_none() {
			
			// char is not found in any fonts
			// In this case, pretend it's in the first font, as it must be drawn
			
			let fccharset = FcCharSetCreate();
			FcCharSetAddChar(fccharset, cur_char as u32);
			if self.fonts[0].pattern_pointer == ptr::null_mut() {
			    /* Refer to the comment in xfont_create for more information. */
			    return Die::stderr("fonts must be loaded from font strings".to_owned());
			}
			
			let fcpattern = FcPatternDuplicate(self.fonts[0].pattern_pointer as *const c_void);
			FcPatternAddCharSet(fcpattern as *mut c_void, FC_CHARSET, fccharset);
			FcPatternAddBool(fcpattern as *mut c_void, FC_SCALABLE, FcTrue);
			FcPatternAddBool(fcpattern as *mut c_void, FC_COLOR, FcFalse);

			FcConfigSubstitute(ptr::null_mut(), fcpattern as *mut c_void, FcMatchPattern);
			let mut result = MaybeUninit::uninit().assume_init(); // XftFontMatch isn't null safe so we need some memory
			let font_match = XftFontMatch(self.dpy, self.screen, fcpattern as *const FcPattern, &mut result);

			FcCharSetDestroy(fccharset);
			FcPatternDestroy(fcpattern);
			
			if font_match != ptr::null_mut() {
			    let mut usedfont = Fnt::new(self, None, font_match)?;
			    
			    if XftCharExists(self.dpy, usedfont.xfont, cur_char as u32) != 0 {
				found_font = Some(self.fonts.len());
				self.fonts.push(usedfont);
			    } else {
				usedfont.free(self.dpy);
				found_font = Some(0);
			    }
			}
		    }
		    // Need to switch fonts
		    // First, take care of the stuff pending print
		    if cur_font.is_some() {
			spool.push((String::from_utf8_unchecked(text.as_bytes()
								[slice_start..slice_end]
								.to_vec()),
				    cur_font));
		    }
		    // Then, set up next thing to print
		    cur_font = found_font;
		    slice_start = slice_end;
		    slice_end += cur_char.len_utf8();
		}
	    }
	    // take care of the remaining slice, if it exists
	    spool.push((String::from_utf8_unchecked(text.as_bytes()
						    [slice_start..slice_end]
						    .to_vec()),
			cur_font));

	    let padded_width = w - self.pseudo_globals.lrpad as u32/2;
	    spool.elipsate(&self, padded_width);
	    while render && spool.width(&self) > padded_width {
		spool.elipse_pop();
	    }
	    
	    let elip_width = spool.elip_width(&self);
	    for (slice, font) in spool.into_iter() {
		// Do early truncation (...)
		self.render(&mut x, &y, &mut w, &h,
			    slice, &font, d, render, invert);
	    }
	    
	    if d != ptr::null_mut() {
		XftDrawDestroy(d);
	    }

	    Ok((x + if render {w} else {0} as i32, elip_width))
	}
    }

    fn render(&self, x: &mut i32, y: &i32, w: &mut u32, h: &u32, text: String, cur_font: &Option<usize>, d: *mut XftDraw, render: bool, invert: bool) {
	if text.len() == 0 {
	    return;
	}
	unsafe {
	    let usedfont = cur_font.map(|i| &self.fonts[i]).unwrap();
	    let font_ref = usedfont;
	    let (substr_width, _) = self.font_getexts(font_ref, text.as_ptr() as *mut c_uchar, text.len() as c_int);
	    if render {
		let ty = *y + (*h as i32 - usedfont.height as i32) / 2 + (*usedfont.xfont).ascent;	
		XftDrawStringUtf8(d, self.scheme[if invert {ColBg} else {ColFg} as usize],  self.fonts[cur_font.unwrap()].xfont, *x, ty, text.as_ptr() as *mut c_uchar, text.len() as c_int);
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
    
    pub fn draw(&mut self) -> CompResult<()> { // drawmenu
	self.pseudo_globals.promptw = if self.config.prompt.len() != 0 {
	    self.textw(Prompt)?
	} else {
	    0
	};
	
	self.setscheme(SchemeNorm);
	self.rect(0, 0, self.w as u32, self.h as u32, true, true); // clear menu

	let mut x = 0;
	
	if self.config.prompt.len() > 0 { // draw prompt
	    self.setscheme(SchemeSel);
	    x = self.text(x, 0, self.pseudo_globals.promptw as c_uint,
			    self.pseudo_globals.bh as u32, self.pseudo_globals.lrpad as u32 / 2, Prompt, false)?.0;
	}

	let matches = Items::draw(self, if self.config.lines > 0 {Vertical} else {Horizontal})?;
	
	/* draw input field */
	let w = if self.config.lines > 0 || self.items.as_mut().unwrap().match_len() == 0
	    || !matches {
	    self.w - x
	} else {
	    if self.config.render_overrun {
		self.textw(Input)?.min(self.w - x)
	    } else {
		self.pseudo_globals.inputw
	    }
	};
	self.setscheme(SchemeNorm);
	let truncated = self.text(x, 0, w as c_uint, self.pseudo_globals.bh as c_uint,
				  self.pseudo_globals.lrpad as c_uint / 2, Input, false)
	    ?.1.map(|u| u + self.pseudo_globals.lrpad/2);
	let inputw = self.textw(Input)?;
	let otherw = self.textw(Other(&self.input.graphemes(true)
				      .skip(self.pseudo_globals.cursor).join("")))?;
	
	let curpos: c_int = inputw - otherw + self.pseudo_globals.lrpad/2 - 1;

	if curpos < truncated.unwrap_or(w - self.pseudo_globals.lrpad/2) {
	    self.setscheme(SchemeNorm);
	    let tallest_font = self.fonts.iter().map(|f| f.height).max().unwrap();
	    self.rect(x + curpos, (self.pseudo_globals.bh - tallest_font) as i32 / 2 + 2, 2, tallest_font - 4, true, false);
	}

	self.map(self.pseudo_globals.win, 0, 0, self.w, self.h);
	Ok(())
    }
    
    pub fn map(&self, win: Window, x: c_int, y: c_int, w: c_int, h: c_int) {
	unsafe {
	    XCopyArea(self.dpy, self.drawable, win, self.gc, x, y, w as u32, h as u32, x, y);
	    XSync(self.dpy, False);
	}
    }

    pub fn textw(&mut self, text: TextOption) -> CompResult<c_int> {
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
*/
