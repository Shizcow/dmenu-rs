use x11::xinerama::{XineramaQueryScreens, XineramaScreenInfo};
use x11::xlib::{Window, XGetInputFocus, PointerRoot, XFree, XQueryTree, XQueryPointer, 
		XGetWindowAttributes, XClassHint, XSetClassHint, CWEventMask, CWBackPixel,
		CWOverrideRedirect, XCreateWindow, VisibilityChangeMask, KeyPressMask,
		ExposureMask, XSetWindowAttributes, XOpenIM,
		XIMStatusNothing, XIMPreeditNothing, XCreateIC, XMapRaised,
		FocusChangeMask, XSelectInput, SubstructureNotifyMask};
use std::ptr;
use std::mem::MaybeUninit;
use libc::{c_char, c_int, c_uint, c_void};

use crate::additional_bindings::xlib::{XNFocusWindow, XNClientWindow, XNInputStyle};
use crate::util::grabfocus;
use crate::config::{Schemes::*, Clrs::*};
use crate::drw::Drw;
use crate::result::*;

#[inline]
fn intersect(x: c_int, y: c_int, w: c_int, h: c_int, r: *mut XineramaScreenInfo) -> c_int {
    unsafe {
	0.max((x+w).min(((*r).x_org+(*r).width) as c_int) - x.max((*r).x_org as c_int)) *
	    0.max((y+h).min(((*r).y_org+(*r).height) as c_int) - y.max((*r).y_org as c_int))
    }
}

impl Drw {
    pub fn setup(&mut self, parentwin: u64, root: u64) -> CompResult<()> {
	unsafe {
	    let mut x: c_int = MaybeUninit::uninit().assume_init();
	    let mut y: c_int = MaybeUninit::uninit().assume_init();
	    
	    let mut ch: XClassHint = XClassHint{
		res_name: (*b"dmenu\0").as_ptr() as *mut c_char,
		res_class: (*b"dmenu\0").as_ptr() as *mut c_char
	    };

	    // appearances are set up in constructor

	    self.pseudo_globals.bh = (self.fonts.iter().map(|f| f.height)
				      .max().unwrap() + 4)
		.max(self.config.render_minheight);
	    self.h = ((self.config.lines + 1) * self.pseudo_globals.bh) as c_int;
	    
	    let mut dws: *mut Window = MaybeUninit::uninit().assume_init();
	    let mut w:  Window = MaybeUninit::uninit().assume_init();
	    let mut dw: Window = MaybeUninit::uninit().assume_init();
	    let mut du: c_uint = MaybeUninit::uninit().assume_init();
	    let mut n:  c_int  = MaybeUninit::uninit().assume_init();
	    let info = if cfg!(feature = "Xinerama") && parentwin == root {
		XineramaQueryScreens(self.dpy, &mut n)
	    } else {
		ptr::null_mut()
	    };
	    if cfg!(feature = "Xinerama") && info != ptr::null_mut() {
		let mut i = 0;
		let mut area = 0;
		let mut di: c_int  = MaybeUninit::uninit().assume_init();
		let mut a;
		let mut pw;
		
		XGetInputFocus(self.dpy, &mut w, &mut di);
		if self.config.mon >= 0 && self.config.mon < n {
		    i = self.config.mon;
		} else if w != root && w != PointerRoot as u64 && w != 0 {
		    /* find top-level window containing current input focus */
		    while {
			pw = w;
			if XQueryTree(self.dpy, pw, &mut dw, &mut w, &mut dws, &mut du) != 0 && dws != ptr::null_mut() {
			    XFree(dws as *mut c_void);
			}
			w != root && w != pw
		    } {} // do-while
		    /* find xinerama screen with which the window intersects most */
		    if XGetWindowAttributes(self.dpy, pw, &mut self.wa) != 0 {
			for j in 0..n {
			    a = intersect(self.wa.x, self.wa.y, self.wa.width, self.wa.height, info.offset(j as isize));
			    if a > area {
				area = a;
				i = j;
			    }
			}
		    }
		}
		/* no focused window is on screen, so use pointer location instead */
		if self.config.mon < 0 && area == 0 && XQueryPointer(self.dpy, root, &mut dw, &mut dw, &mut x, &mut y, &mut di, &mut di, &mut du) != 0 {
		    for j in 0..n {
			i = j; // this is here to bypass rust's shadowing rules in an efficient way
			if intersect(x, y, 1, 1, info.offset(i as isize)) != 0 {
			    break;
			}
		    }
		}
		x = (*info.offset(i as isize)).x_org as c_int;
		y = (*info.offset(i as isize)).y_org as c_int + (if self.config.topbar {0} else {(*info.offset(i as isize)).height as c_int - self.h as c_int});
		self.w = (*info.offset(i as isize)).width as c_int;
		XFree(info as *mut c_void);
	    } else {
		if XGetWindowAttributes(self.dpy, parentwin, &mut self.wa) == 0 {
		    return Die::stderr(format!("could not get embedding window attributes: 0x{:?}", parentwin));
		}
		x = 0;
		y = if self.config.topbar {
		    0
		} else {
		    self.wa.height - self.h as c_int
		};
		self.w = self.wa.width;
	    }

	    let mut swa: XSetWindowAttributes = MaybeUninit::uninit().assume_init();
	    swa.override_redirect = true as i32;
	    swa.background_pixel = (*self.pseudo_globals.schemeset[SchemeNorm as usize][ColBg as usize]).pixel;
	    swa.event_mask = ExposureMask | KeyPressMask | VisibilityChangeMask;
	    self.pseudo_globals.win =
		XCreateWindow(self.dpy, parentwin, x, y, self.w as u32,
			      self.h as u32, 0, 0,
			      0, ptr::null_mut(),
			      CWOverrideRedirect | CWBackPixel | CWEventMask, &mut swa);
	    XSetClassHint(self.dpy, self.pseudo_globals.win, &mut ch);

	    /* input methods */
	    let xim = XOpenIM(self.dpy, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
	    if xim == ptr::null_mut() {
		return Die::stderr("XOpenIM failed: could not open input device".to_owned());
	    }

	    
	    self.pseudo_globals.xic = XCreateIC(xim, XNInputStyle,
						XIMPreeditNothing | XIMStatusNothing,
						XNClientWindow, self.pseudo_globals.win,
						XNFocusWindow, self.pseudo_globals.win,
						ptr::null_mut::<c_void>());
	    // void* makes sure the value is large enough for varargs to properly stop
	    // parsing. Any smaller and it will skip over, causing a segfault

	    
	    XMapRaised(self.dpy, self.pseudo_globals.win);

	    if self.config.embed != 0 {
		XSelectInput(self.dpy, parentwin, FocusChangeMask | SubstructureNotifyMask);
		if XQueryTree(self.dpy, parentwin, &mut dw, &mut w, &mut dws, &mut du) != 0 && dws != ptr::null_mut() {
		    for i in 0..du {
			if *dws.offset(i as isize) == self.pseudo_globals.win {
			    break;
			}
			XSelectInput(self.dpy, *dws.offset(i as isize), FocusChangeMask);
		    }
		    XFree(dws as *mut c_void);
		}
		grabfocus(self)?;
	    }

	    self.draw()
	}
    }
}
