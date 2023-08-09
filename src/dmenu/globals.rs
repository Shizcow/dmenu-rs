use crate::config::Schemes::*;
use libc::c_int;
use std::ptr;
use x11::xft::XftColor;
use x11::xlib::{Window, XIC};

#[derive(Debug)]
pub struct PseudoGlobals {
    pub promptw: c_int,
    pub inputw: c_int,
    pub lrpad: c_int,
    pub schemeset: [[*mut XftColor; 2]; SchemeLast as usize],
    pub bh: u32,
    pub win: Window,
    pub cursor: usize,
    pub xic: XIC,
}

impl Default for PseudoGlobals {
    fn default() -> Self {
        Self {
            promptw: 0,
            inputw: 0,
            schemeset: [[ptr::null_mut(); 2]; SchemeLast as usize],
            lrpad: 0,
            bh: 0,
            win: 0,
            cursor: 0,
            xic: ptr::null_mut(),
        }
    }
}
