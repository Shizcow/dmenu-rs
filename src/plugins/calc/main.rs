use overrider::*;
use rink::*;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::drw::Drw;
use crate::item::Item;

#[override_flag(flag = calc)]
impl Drw {
    pub fn gen_matches(&mut self) -> Result<Vec<Item>, String> {
	let mut ctx = load().unwrap();
	let eval = self.config.prompt.clone() + " " + &self.input;
	if let Ok(evaluated) = one_line(&mut ctx, &eval) {
	    Ok(vec![Item::new(evaluated, false, self)?])
	} else {
	    Ok(vec![])
	}
    }
    pub fn dispose(&mut self, _output: String, recommendation: bool) -> Result<bool, String> {
	let mut ctx = load().unwrap();
	let eval = self.config.prompt.clone() + " " + &self.input;
	let output = if let Ok(evaluated) = one_line(&mut ctx, &eval) {
	    evaluated
	} else {
	    return Ok(false)
	};
	
	self.input = "".to_owned();
	self.pseudo_globals.cursor = 0;
	self.config.prompt = output.clone();
	if output.len() > 0 {
	    // Wow making sure keyboard content sticks around after exit is a pain in the neck
	    
	    let mut child = Command::new("xclip")
		.arg("-sel")
		.arg("clip")
		.stdin(Stdio::piped())
		.spawn()
		.map_err(|_| "Failed to spawn child process".to_owned())?;

	    child.stdin.as_mut().ok_or("Failed to open stdin".to_owned())?
	    .write_all(output.as_bytes()).map_err(|_| "Failed to write to stdin".to_owned())?;
	}
	self.draw()?;
	Ok(!recommendation)
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
