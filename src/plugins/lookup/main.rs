use overrider::*;
use std::process::Command;

use crate::clapflags::CLAP_FLAGS;
use crate::config::ConfigDefault;
use crate::drw::Drw;
use crate::result::*;
use itertools::Itertools;

mod engines;
use engines::ENGINES;

// Format engine as prompt
// eg "ddg" -> "[Search ddg]"
fn create_search_input(engine: &str) -> CompResult<String> {
    // fail early if engine is wrong
    match ENGINES.get(engine) {
        Some(_) => Ok(format!("[Search {}]", engine)),
        None => {
            return Err(Die::Stderr(format!(
                "Invalid search search engine {}. Valid options are: {}",
                engine,
                ENGINES.keys().map(|e| format!("\"{}\"", e)).join(", ")
            )))
        }
    }
}

// Take the output of create_search_input as prompt
// It's not very clean but hey it works
fn do_dispose(output: &str, prompt: &str) -> CompResult<()> {
    let mut engine: String = prompt.chars().skip("[Search ".len()).collect();
    engine.pop();

    // just unwrap since the check was performed before
    let search_prompt = ENGINES.get(engine.as_str())
	.unwrap().to_string().replace("%s", output);

    // TODO: consider user defined open command for cross-platform awareness
    Command::new("xdg-open")
        .arg(search_prompt)
        .spawn()
        .map_err(|_| Die::Stderr("Failed to spawn child process".to_owned()))?;
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

#[override_flag(flag = listEngines, priority = 2)]
impl Drw {
    pub fn format_stdin(&mut self, _: Vec<String>) -> CompResult<Vec<String>> {
        Err(Die::Stdout(ENGINES.keys().join("\n")))
    }
}
#[override_flag(flag = listEngines, priority = 2)]
impl ConfigDefault {
    pub fn nostdin() -> bool {
        true // if called with --list-engines, takes no stdin (only prints)
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
