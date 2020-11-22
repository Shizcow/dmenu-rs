use overrider::*;
use std::io::Write;
use std::process::Command;

use crate::drw::Drw;
use crate::result::*;
use crate::config::{ConfigDefault, DefaultWidth};

#[override_flag(flag = lookup)]
impl Drw {
    // I know I need to implement format_stdin and put the logic to select the url there,
    // but I don't know how to access it on dispose
    pub fn format_input(&self) -> CompResult<String> {
	Ok(format!("[Search DDG]: {}",self.input))
    }
    pub fn dispose(&mut self, _output: String, recommendation: bool) -> CompResult<bool> {
	let eval = format!("https://duckduckgo.com/{}", self.input);
	self.input = "".to_owned();
	self.pseudo_globals.cursor = 0;
	if eval.len() > 0 {
	    let mut child = Command::new("xdg-open")
		.arg(eval.clone())
		.spawn()
		.map_err(|_| Die::Stderr("Failed to spawn child process".to_owned()))?;

	    child.stdin.as_mut().ok_or(Die::Stderr("Failed to open stdin of child process"
						   .to_owned()))?
	    .write_all(eval.as_bytes())
		.map_err(|_| Die::Stderr("Failed to write to stdin of child process"
					 .to_owned()))?;
	}
	self.draw()?;
	Ok(!recommendation)
    }
}

#[override_flag(flag = lookup)]
impl ConfigDefault {
    pub fn nostdin() -> bool {
        // setting this to false makes it get stuck
	true
    }
    pub fn render_flex() -> bool {
	true
    }
    pub fn render_default_width() -> DefaultWidth {
	DefaultWidth::Custom(25)
    }
}

