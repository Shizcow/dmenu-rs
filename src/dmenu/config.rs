#[derive(Debug)]
pub struct Config {
    // pub lines: c_uint,
    // pub topbar: bool,
    // pub prompt: String,
    // pub promptw: c_int,
    // pub fontstrings: Vec<String>,
    pub fast: bool,
    // pub embed: Window,
    // pub case_sensitive: bool,
    // pub mon: c_int,
    // pub colors: [[[u8; 8]; 2]; SchemeLast as usize],
    // pub render_minheight: u32,
    // pub render_overrun: bool,
    // pub render_flex: bool,
    // pub render_rightalign: bool,
    // pub render_default_width: DefaultWidth,
    pub nostdin: bool,
}

//pub struct ConfigDefault{}

impl Default for Config {
    fn default() -> Self {
	Self{
	    fast:                 false,
	    nostdin:              false,
	}
    }
}
