use x11::xlib::{Window, XIC};
use x11::xft::XftColor;
use libc::c_int;
use crate::config::Schemes::*;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct PseudoGlobals {
    pub promptw: c_int,
    pub inputw: c_int,
    pub lrpad: c_int,
    pub schemeset: [[*mut XftColor; 2]; SchemeLast as usize],
    pub mon: c_int,
    pub bh: c_int,
    pub win: Window,
    pub embed: Window,
    pub cursor: usize,
    pub xic: XIC,
}

impl Default for PseudoGlobals {
    fn default() -> Self {
	unsafe {
	    Self {
		promptw:   MaybeUninit::uninit().assume_init(),
		inputw:    0,
		schemeset: MaybeUninit::uninit().assume_init(),
		lrpad:     MaybeUninit::uninit().assume_init(),
		mon:       -1,
		bh:         MaybeUninit::uninit().assume_init(),
		win:        MaybeUninit::uninit().assume_init(),
		embed:      0,
		cursor:     0,
		xic:        MaybeUninit::uninit().assume_init(),
	    }
	}
    }
}
