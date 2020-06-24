#[allow(unused_imports)]
use crate::clapflags::CLAP_FLAGS;
use crate::drw::Drw;
use crate::item::{Item, MatchCode};

use overrider::*;
#[allow(unused_imports)]
use regex::{Regex, RegexBuilder};


#[default]
impl Drw {
    pub fn format_input(&self) -> String {
	self.input.clone()
    }
    
    pub fn gen_matches(&mut self) -> Result<Vec<Item>, String> {
	let re = match RegexBuilder::new(&regex::escape(&self.input))
	    .case_insensitive(!self.config.case_sensitive)
	    .build() {
		Ok(re) => re,
		Err(_) => return Err(format!("Could not build regex")),
	    };
	let mut exact:     Vec<Item> = Vec::new();
	let mut prefix:    Vec<Item> = Vec::new();
	let mut substring: Vec<Item> = Vec::new();
	for item in &self.items.as_mut().unwrap().data {
	    /*match item.matches(&re) {
		MatchCode::Exact => */exact.push(item.clone())/*,
		MatchCode::Prefix => prefix.push(item.clone()),
		MatchCode::Substring => substring.push(item.clone()),
		MatchCode::None => {}
	    }*/
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
