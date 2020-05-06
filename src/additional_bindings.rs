// A few additional bindings are needed from fondconfig.h
// Because servo-fontconfig provides very clean bindings for everything,
// only the bindings not included there are mapped here
mod raw { // TODO: remove pub
    pub mod main {
	#![allow(non_upper_case_globals)]
	#![allow(non_camel_case_types)]
	#![allow(non_snake_case)]
	include!(concat!(env!("OUT_DIR"), "/bindings_main.rs"));
    }
    pub mod xlib {
	#![allow(non_upper_case_globals)]
	#![allow(non_camel_case_types)]
	#![allow(non_snake_case)]
	include!(concat!(env!("OUT_DIR"), "/bindings_xlib.rs"));
    }
}
pub mod fontconfig {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    use super::raw::main;
    pub const FcTrue:  main::FcBool  = main::FcTrue  as main::FcBool;
    pub const FcFalse: main::FcBool  = main::FcFalse as main::FcBool;
    pub const FC_SCALABLE: *const i8 = main::FC_SCALABLE.as_ptr() as *const i8;
    pub const FC_CHARSET:  *const i8 = main::FC_CHARSET.as_ptr()  as *const i8;
    pub const FC_COLOR:    *const i8 = main::FC_COLOR.as_ptr()    as *const i8;
}
#[cfg(feature = "Xinerama")]
pub mod Xinerama { // TODO: do we need this here?
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    use super::raw::main;
}
pub mod xlib {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    use super::raw::xlib;
    pub use xlib::{XNInputStyle, XNClientWindow, XNFocusWindow};
}
