#[allow(unused_imports)]
use crate::clapflags::CLAP_FLAGS;
use overrider::*;

use crate::drw::Drw;

use crate::item::Item;

#[default]
impl Drw {
    pub fn format_input(&self) -> String {
	self.input.clone()
    }
}

/*
#[default]
impl Drw {
    pub fn item_matches(&mut self) -> Vec<Item> {
	self.items.as_mut().unwrap().data_matches.iter()
	    .map(|s| s.clone()).collect()
    }
}*/
