use x11::xlib::Window;
use std::mem::MaybeUninit;
use libc::{c_int, c_uint};
use std::str::FromStr;


pub enum Schemes { SchemeNorm, SchemeSel, SchemeOut, SchemeLast }
pub enum Clrs    { ColFg, ColBg }
pub use Schemes::*;
pub use Clrs::*;

#[derive(Debug)]
pub enum InputFlex {
    Strict,
    Flex,
    Overrun,
}

impl FromStr for InputFlex {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
	match s {
	    "0" => Ok(Self::Strict),
	    "1" => Ok(Self::Flex),
	    "2" => Ok(Self::Overrun),
	    _ => Err(format!("-x: Flex value invalid -- see help for more details")),
	}
    }
}

#[derive(Debug)]
pub struct Config {
    pub lines: c_uint,
    pub topbar: bool,
    pub prompt: String,
    pub promptw: c_int,
    pub default_font: String,
    pub fast: bool,
    pub embed: Window,
    pub case_sensitive: bool,
    pub mon: c_int,
    pub colors: [[[u8; 8]; 2]; SchemeLast as usize],
    pub input_flex: InputFlex,
}

impl Default for Config {
    fn default() -> Self {
	unsafe {
	    Self{
		lines: 0,
		topbar: true,
		prompt: "".to_string(),
		promptw: MaybeUninit::uninit().assume_init(),
		default_font: "monospace:size=10\0".to_string(),
		fast: false,
		embed: 0,
		case_sensitive: true,
		mon: -1,
		colors: {
		    /*     fg         bg       */
		    let mut arr = [[[0; 8]; 2]; SchemeLast as usize]; // init is optimized out
		    arr[SchemeNorm as usize] = [*b"#bbbbbb\0", *b"#222222\0"];
		    arr[SchemeSel  as usize] = [*b"#eeeeee\0", *b"#005577\0"];
		    arr[SchemeOut  as usize] = [*b"#000000\0", *b"#00ffff\0"];
		    arr
		},
		input_flex: InputFlex::Strict,
	    }
	}
    }
}
