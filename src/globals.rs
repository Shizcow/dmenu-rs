use x11::xlib::Window;
use libc::c_int;
use crate::drw::Clr;
use crate::config::Schemes::*;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct PseudoGlobals {
    pub promptw: c_int,
    pub lrpad: c_int,
    pub schemeset: [[*mut Clr; 2]; SchemeLast as usize], // replacement for "scheme"
    pub mon: c_int,
    pub mw: c_int,
    pub bh: c_int,
    pub mh: c_int,
    pub win: Window,
    pub embed: Window,
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
		bh:         MaybeUninit::uninit().assume_init(),
		mh:         MaybeUninit::uninit().assume_init(),
		win:        MaybeUninit::uninit().assume_init(),
		embed:      0,
	    }
	}
    }
}