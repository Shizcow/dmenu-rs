mod raw {
    pub mod main {
	#![allow(non_upper_case_globals)]
	#![allow(non_camel_case_types)]
	#![allow(non_snake_case)]
	#![allow(unused)]
	include!(concat!(env!("BUILD_DIR"), "/bindings_main.rs"));
    }
    pub mod xlib {
	#![allow(non_upper_case_globals)]
	#![allow(non_camel_case_types)]
	#![allow(non_snake_case)]
	#![allow(unused)]
	include!(concat!(env!("BUILD_DIR"), "/bindings_xlib.rs"));
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
    pub const FC_FAMILY:   *mut   i8 = main::FC_FAMILY.as_ptr()   as *mut   i8;
}
pub mod xlib {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    use super::raw::xlib;
    pub use xlib::{XNInputStyle, XNClientWindow, XNFocusWindow};
}
