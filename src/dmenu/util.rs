use std::io::{self, BufRead};
use crate::result::*;

pub fn readstdin() -> CompResult<Vec<String>> {
    let mut items: Vec<String> = Vec::new();
    for line in io::stdin().lock().lines() {
	match line {
	    Ok(l) => items.push(l),
	    Err(e) => return Die::stderr(format!("Could not read from stdin: {}", e)),
	}
    }
    Ok(items)
}
