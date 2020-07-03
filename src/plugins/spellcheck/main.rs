use overrider::*;

use ispell::{SpellLauncher};
use std::process::{Command, Stdio};
use std::io::Write;

use crate::drw::Drw;
use crate::item::Item;

#[override_flag(flag = spellcheck)]
impl Drw {
    pub fn gen_matches(&mut self) -> Result<Vec<Item>, String> {
	let checker = SpellLauncher::new()
	    .aspell()
            .launch();

	self.input = self.input.split(" ").nth(0).unwrap_or("").to_owned();
	self.pseudo_globals.cursor = self.input.chars().count();
	
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
    pub fn dispose(&mut self, output: String, recommendation: bool) -> Result<bool, String> {
	if output.len() > 0 {
	    let mut child = Command::new("xclip")
		.arg("-sel")
		.arg("clip")
		.stdin(Stdio::piped())
		.spawn()
		.map_err(|_| "Failed to spawn child process".to_owned())?;

	    child.stdin.as_mut().ok_or("Failed to open stdin".to_owned())?
	    .write_all(output.as_bytes()).map_err(|_| "Failed to write to stdin".to_owned())?;
	}
	Ok(recommendation)
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
	DefaultWidth::Custom(10)
    }
}
