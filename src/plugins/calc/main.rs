use overrider::*;
use rink::*;

use crate::drw::Drw;
use crate::item::Item;

#[override_flag(flag = calc)]
impl Drw {
    pub fn gen_matches(&mut self) -> Result<Vec<Item>, String> {
	let mut ctx = load().unwrap();
	if let Ok(evaluated) = one_line(&mut ctx, &self.input) {
	    Ok(vec![Item::new(evaluated, false, self)?])
	} else {
	    Ok(vec![])
	}
    }
}

use crate::config::{ConfigDefault, DefaultWidth};
#[override_flag(flag = calc)]
impl ConfigDefault {
    pub fn nostdin() -> bool {
	true
    }
    pub fn render_flex() -> bool {
	true
    }
    pub fn render_default_width() -> DefaultWidth {
	DefaultWidth::Custom(25)
    }
}
