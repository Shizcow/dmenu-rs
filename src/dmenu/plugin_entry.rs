#[allow(unused_imports)]
use crate::clapflags::CLAP_FLAGS;
use crate::drw::Drw;
use crate::item::{Item, MatchCode};

use overrider::*;
#[allow(unused_imports)]
use regex::{Regex, RegexBuilder};


#[default]
impl Drw {
    /**
     * Every time the input is drawn, how should it be presented?
     * Does it need additional processing?
     */
    pub fn format_input(&self) -> String {
	self.input.clone()
    }

    /**
     * What to do when printing to stdout / program termination?
     * 
     * Args:
     * - output: what's being processed
     * - recommendation: is exiting recommended? C-Enter will not normally exit
     * 
     * Returns - true if program should exit
     */
    pub fn dispose(&mut self, output: String, recommendation: bool) -> Result<bool, String> {
	println!("{}", output);
	Ok(recommendation)
    }
    
    pub fn gen_matches(&mut self) -> Result<Vec<Item>, String> {
	let re = RegexBuilder::new(&regex::escape(&self.input))
	    .case_insensitive(!self.config.case_sensitive)
	    .build().map_err(|_| format!("Could not build regex"))?;
	let mut exact:     Vec<Item> = Vec::new();
	let mut prefix:    Vec<Item> = Vec::new();
	let mut substring: Vec<Item> = Vec::new();
	for item in &self.items.as_mut().unwrap().data {
	    match item.matches(&re) {
		MatchCode::Exact => exact.push(item.clone()),
		MatchCode::Prefix => prefix.push(item.clone()),
		MatchCode::Substring => substring.push(item.clone()),
		MatchCode::None => {}
	    }
	}
	exact.reserve(prefix.len()+substring.len());
	for item in prefix { // extend is broken for pointers
	    exact.push(item);
	}
	for item in substring {
	    exact.push(item);
	}
	Ok(exact)
    }
}

use crate::config::DefaultWidth;
use crate::config::Schemes::*;
use crate::config::ConfigDefault;
#[default]
impl ConfigDefault {
    pub fn lines() -> u32 {
	0
    }
    pub fn topbar() -> bool {
	true
    }
    pub fn prompt() -> String {
	"".to_string()
    }
    pub fn default_font() -> String {
	"monospace:size=10\0".to_string()
    }
    pub fn fast() -> bool {
	false
    }
    pub fn embed() -> u64 {
	0
    }
    pub fn case_sensitive() -> bool {
	true
    }
    pub fn mon() -> i32 {
	-1
    }
    pub fn colors() -> [[[u8; 8]; 2]; SchemeLast as usize] {
	/*                         [  fg             bg         ]*/
	let mut arr = [[[0; 8]; 2]; SchemeLast as usize];
	arr[SchemeNorm as usize] = [*b"#bbbbbb\0", *b"#222222\0"];
	arr[SchemeSel  as usize] = [*b"#eeeeee\0", *b"#005577\0"];
	arr[SchemeOut  as usize] = [*b"#000000\0", *b"#00ffff\0"];
	arr
    }
    pub fn nostdin() -> bool {
	false
    }
    pub fn render_overrun() -> bool {
	false
    }
    pub fn render_flex() -> bool {
	false
    }
    pub fn render_rightalign() -> bool {
	false
    }
    pub fn render_default_width() -> DefaultWidth {
	DefaultWidth::Items
    }
}
