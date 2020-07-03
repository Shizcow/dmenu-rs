use overrider::*;

use crate::clapflags::CLAP_FLAGS;

use ispell::{SpellLauncher};

use crate::drw::Drw;
use crate::item::Item;

#[override_flag(flag = spellcheck)]
impl Drw {
    pub fn gen_matches(&mut self) -> Result<Vec<Item>, String> {
	let checker = SpellLauncher::new()
	    .aspell()
            .launch();
	
	match checker {
            Ok(mut checker) => {
		match checker.check(&self.input) {
		    Ok(mut res) => {
			if res.is_empty() {
			    Ok(vec![Item::new(self.input.clone(), false, self)?])
			} else {
			    let mut ret = Vec::new();
			    for word in res.swap_remove(0).suggestions.into_iter() {
				ret.push(Item::new(word, false, self)?);
			    }
			    Ok(ret)
			}
		    },
		    Err(err) => Err(format!("Error: could not run aspell: {}", err))
		}
            },
            Err(err) => Err(format!("Error: could not start aspell: {}", err))
	}
    }
}

use crate::config::{ConfigDefault, DefaultWidth};
#[override_flag(flag = spellcheck)]
impl ConfigDefault {
    pub fn nostdin() -> bool {
	true
    }
    pub fn render_flex() -> bool {
	true
    }
    pub fn render_default_width() -> DefaultWidth {
	DefaultWidth::Custom(15)
    }
}
