use overrider::*;
use rink_core::{one_line, simple_context, Context};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use async_std::prelude::*;
use async_std::task::block_on;

use crate::drw::Drw;
use crate::item::Item;
use crate::result::*;

lazy_static::lazy_static! {
    static ref CTX: Mutex<Context> = Mutex::new(simple_context().unwrap());
}

async fn timed_eval(expr: String) -> Result<String, String> {
    async {
	async_std::task::spawn(async {
	    let xpr = expr;
	    one_line(&mut CTX.lock().unwrap(), &xpr)
	}).await
    }.timeout(Duration::from_millis(250))
	.await.unwrap_or(Ok("Calculation Timeout".to_string()))	
}

#[override_flag(flag = calc)]
impl Drw {
    pub fn gen_matches(&mut self) -> CompResult<Vec<Item>> {
	let eval = self.config.prompt.clone() + " " + &self.input;
	if let Ok(evaluated) = block_on(timed_eval(eval)) {
	    Ok(vec![Item::new(evaluated, false, self)?])
	} else {
	    Ok(vec![])
	}
    }
    pub fn dispose(&mut self, _output: String, recommendation: bool) -> CompResult<bool> {
	let eval = self.config.prompt.clone() + " " + &self.input;
	let output = if let Ok(evaluated) = block_on(timed_eval(eval)) {
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
		.map_err(|_| Die::Stderr("Failed to spawn child process".to_owned()))?;

	    child.stdin.as_mut().ok_or(Die::Stderr("Failed to open stdin of child process"
						   .to_owned()))?
	    .write_all(output.as_bytes())
		.map_err(|_| Die::Stderr("Failed to write to stdin of child process"
					 .to_owned()))?;
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
