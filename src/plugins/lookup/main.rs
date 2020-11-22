use overrider::*;
use crate::clapflags::CLAP_FLAGS;

use crate::drw::Drw;
use crate::result::*;
use crate::config::{ConfigDefault};


// Turns short description into a prompt
// eg "D" -> "[Search DDG]"
fn create_search_input(engine: &str) -> CompResult<String> {
    Ok(format!("[Search {}]", match engine {
	"D" => "DDG",
	// add more engines here if you want
	_ => return Err(Die::Stderr("invalid engine".to_string()))
    }))
}

// Takes the output of create_search_input as prompt
// It's not very clean but hey it works
fn do_dispose(output: &str, prompt: &str) -> CompResult<()> {
    // Extract "ENGINE_LONG" from "[Search ENGINE_LONG]"
    let mut engine: String = prompt.chars().skip("[Search ".len()).collect();
    engine.pop();
    
    println!("engine: {}, searchterm: {}", engine, output);

    // xdg logic goes here

    Ok(())
}

// Important: engine must become before lookup. It's a bug in overrider.
#[override_flag(flag = engine, priority = 2)]
impl Drw {
    pub fn dispose(&mut self, output: String, recommendation: bool) -> CompResult<bool> {
	do_dispose(&output, &self.config.prompt)?;
	Ok(recommendation)
    }
    pub fn format_stdin(&mut self, _lines: Vec<String>) -> CompResult<Vec<String>> {
	self.config.prompt = create_search_input(CLAP_FLAGS.value_of("engine").unwrap())?;
	Ok(vec![]) // turns into prompt
    }
}

#[override_flag(flag = engine, priority = 2)]
impl ConfigDefault {
    pub fn nostdin() -> bool {
	true // if called with --engine ENGINE, takes no stdin
    }
}

#[override_flag(flag = lookup, priority = 1)]
impl Drw {
    pub fn dispose(&mut self, output: String, recommendation: bool) -> CompResult<bool> {
	do_dispose(&output, &self.config.prompt)?;
	Ok(recommendation)
    }
    pub fn format_stdin(&mut self, lines: Vec<String>) -> CompResult<Vec<String>> {
	self.config.prompt = create_search_input(&lines[0])?;
	Ok(vec![]) // turns into prompt
    }
}

#[override_flag(flag = lookup, priority = 1)]
impl ConfigDefault {
    pub fn nostdin() -> bool {
	false // if called without --engine, takes stdin
    }
}
