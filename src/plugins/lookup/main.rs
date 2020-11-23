use overrider::*;
use std::process::Command;

use crate::clapflags::CLAP_FLAGS;
use crate::config::ConfigDefault;
use crate::drw::Drw;
use crate::result::*;

use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

// the user can simply define here more pairs of engine => url
static ENGINES: Lazy<Mutex<HashMap<String, &'static str>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("crates".to_string(), "https://crates.io/crates/");
    m.insert("github".to_string(), "https://github.com/search?q=");
    m.insert("rust".to_string(), "https://doc.rust-lang.org/std/?search=");
    m.insert(
        "archwiki".to_string(),
        "https://wiki.archlinux.org/index.php?search=",
    );
    m.insert("ddg".to_string(), "https://duckduckgo.com/");
    m.insert(
        "english".to_string(),
        "https://www.merriam-webster.com/dictionary/",
    );
    m.insert("docs".to_string(), "https://docs.rs/");
    Mutex::new(m)
});

// Format engine as prompt
// eg "ddg" -> "[Search ddg]"
fn create_search_input(engine: &str) -> CompResult<String> {
    let engines = ENGINES.lock().unwrap();
    // fail early if engine is wrong
    match engines.get(&engine.to_string()) {
        Some(_) => Ok(format!("[Search {}]", engine)),
        None => {
            return Err(Die::Stderr(format!(
                "Invalid search search engine {}. Valid options are: {:?}",
                engine,
                engines.keys()
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
    let search_prompt = format!(
        "{}{}",
        ENGINES.lock().unwrap().get(&engine).unwrap(),
        output
    );

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