use x11::xlib::Window;
use std::mem::MaybeUninit;
use libc::{c_int, c_uint};
use std::str::FromStr;

pub enum Schemes { SchemeNorm, SchemeSel, SchemeOut, SchemeLast }
pub enum Clrs    { ColFg, ColBg }
pub use Schemes::*;
pub use Clrs::*;

#[derive(Debug, PartialEq)]
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

pub struct ConfigDefault{}

impl Default for Config {
    fn default() -> Self {
	unsafe {
	    Self{
		lines: ConfigDefault::lines(),
		topbar: ConfigDefault::topbar(),
		prompt: ConfigDefault::prompt(),
		promptw: MaybeUninit::uninit().assume_init(),
		default_font: ConfigDefault::default_font(),
		fast: ConfigDefault::fast(),
		embed: ConfigDefault::embed(),
		case_sensitive: ConfigDefault::case_sensitive(),
		mon: ConfigDefault::mon(),
		colors: ConfigDefault::colors(),
		input_flex: ConfigDefault::input_flex(),
	    }
	}
    }
}
