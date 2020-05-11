mod raw {
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
pub mod keysym {
    use libc::*;
    use x11::keysym::*;
    use x11::xlib::*;
    pub const X_LOOKUP_CHARS: c_int = XLookupChars;
    pub const X_LOOKUP_KEYSYM: c_int = XLookupKeySym;
    pub const X_LOOKUP_BOTH: c_int = XLookupBoth;
    pub const SELECTION_NOTIFY: c_int = SelectionNotify;
    pub const VISIBILITY_NOTIFY: c_int = VisibilityNotify;
    pub const FOCUS_IN: c_int = FocusIn;
    pub const EXPOSE: c_int = Expose;
    pub const KEY_PRESS: c_int = KeyPress;
    pub const DESTROY_NOTIFY: c_int = DestroyNotify;
}
