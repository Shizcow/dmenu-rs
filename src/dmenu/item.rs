use libc::c_int;
use regex::{Regex, RegexBuilder};

use crate::drw::{Drw, TextOption::*};
use crate::config::Schemes::*;

pub enum MatchCode {Exact, Prefix, Substring, None}
#[derive(Debug)]
pub enum Direction {Vertical, Horizontal}
pub use Direction::*;

#[derive(Debug, Clone)]
pub struct Item { // dmenu entry
    pub text: String,
    pub out: bool,
    pub width: c_int,
}

impl Item {
    pub fn new(text: String, out: bool, drw: &mut Drw) -> Result<Self, String> {
	Ok(Self{out, width: match drw.textw(Other(&text)) {
	    Ok(w) => w,
	    Err(err) => return Err(err),
	}, text})
    }
    pub fn draw(&self, x: c_int, y: c_int, w: c_int, drw: &mut Drw) -> Result<c_int, String> {
	drw.text(x, y, w as u32, drw.pseudo_globals.bh as u32, drw.pseudo_globals.lrpad as u32/2, Other(&self.text), false)
    }
    pub fn matches(&self, re: &Regex) -> MatchCode {
	match re.find_iter(&self.text)
	    .nth(0).map(|m| (m.start(), m.end()))
	    .unwrap_or((1,0)) {
		(1, 0) => MatchCode::None, // don't expect zero length matches...
		(0, end) => //                unless search is empty
		    if end == self.text.len() {
			MatchCode::Exact
		    } else {
			MatchCode::Prefix
		    },
		_ => MatchCode::Substring,
	    }
    }
}

#[derive(Debug)]
pub struct Items {
    pub data: Vec<Item>,
    pub cached_partitions: Vec<Vec<Item>>, // seperated into screens
    pub curr: usize,
}

impl Items {
    pub fn new(data: Vec<Item>) -> Self {
	Self{data, cached_partitions: Vec::new(), curr: 0}
    }
    pub fn match_len(&self) -> usize {
	self.cached_partitions.len()
    }
    pub fn draw(drw: &mut Drw, direction: Direction) -> Result<(), String> { // gets an apropriate vec of matches
	let items_to_draw = match Self::gen_matches(drw) {
	    Ok(items) => items,
	    Err(err) => return Err(err),
	};
	let matched_partitions = match Self::partition_matches(items_to_draw, &direction, drw) {
	    Ok(partitions) => partitions,
	    Err(err) => return Err(err),
	};

	if matched_partitions.len() == 0 {
	    return Ok(()); // nothing to draw
	}
	
	let rangle = ">".to_string();
	let rangle_width = match drw.textw(Other(&rangle)) {
	    Ok(w) => w,
	    Err(err) => return Err(err),
	};
	let langle = "<".to_string();
	let langle_width = match drw.textw(Other(&langle)) {
	    Ok(w) => w,
	    Err(err) => return Err(err),
	};

	let mut coord = match direction {
	    Horizontal => drw.pseudo_globals.promptw + drw.pseudo_globals.inputw,
	    Vertical => drw.pseudo_globals.bh,
	};
	
	let (partition_i, partition) = {
	    let mut partition_i = drw.items.as_mut().unwrap().curr;
	    let mut partition = 0;
	    for p in &matched_partitions {
		if partition_i >= p.len() {
		    partition_i -= p.len();
		    partition += 1;
		} else {
		    break;
		}
	    }
	    (partition_i, partition)
	};

	if let Horizontal = direction {
	    if partition > 0 { // draw langle if required
		drw.setscheme(SchemeNorm);
		match drw.text(coord, 0, langle_width as u32, drw.pseudo_globals.bh as u32, drw.pseudo_globals.lrpad as u32/2, Other(&langle), false) {
		    Ok(computed_width) => coord = computed_width,
		    Err(err) => return Err(err),
		}
	    } else {
		coord += langle_width;
	    }
	}

	
	for index in 0..matched_partitions[partition].len() {
	    if index == partition_i {
		drw.setscheme(SchemeSel);
	    } else if matched_partitions[partition][index].out {
		drw.setscheme(SchemeOut);
	    } else {   
		drw.setscheme(SchemeNorm);
	    }
	    match direction {
		Horizontal => {
		    match matched_partitions[partition][index]
			.draw(coord, 0, matched_partitions[partition][index]
			      .width.min(drw.w - coord - rangle_width), drw) { 
			    Ok(computed_width) => coord = computed_width,
			    Err(err) => return Err(err),
			}
		    if partition+1 < matched_partitions.len() { // draw rangle
			drw.setscheme(SchemeNorm);
			if let Err(err) = drw.text(drw.w - rangle_width, 0, rangle_width as u32, drw.pseudo_globals.bh as u32, drw.pseudo_globals.lrpad as u32/2, Other(&rangle), false) {
			    return Err(err);
			}
		    }
		},
		Vertical => {
		    match matched_partitions[partition][index].draw(0, coord, drw.w, drw) {
			Ok(_) => coord += drw.pseudo_globals.bh,
			Err(err) => return Err(err),
		    }
		}
	    }	    
	}
	
	drw.items.as_mut().unwrap().cached_partitions = matched_partitions;
	
	Ok(())
    }
    fn partition_matches(input: Vec<Item>, direction: &Direction, drw: &mut Drw) -> Result<Vec<Vec<Item>>, String> { // matches come in, partitions come out
	match direction {
	    Horizontal => {
		let mut partitions = Vec::new();
		let mut partition = Vec::new();
		let rangle_width = match drw.textw(Other(&">".to_string())) {
		    Ok(w) => w,
		    Err(err) => return Err(err),
		};
		let langle_width = match drw.textw(Other(&"<".to_string())) {
		    Ok(w) => w,
		    Err(err) => return Err(err),
		};
		let mut x = drw.pseudo_globals.promptw + drw.pseudo_globals.inputw
		    + langle_width;
		let mut item_iter = input.into_iter().peekable();
		while let Some(item) = item_iter.next() {
		    x += item.width;
		    if x > {
			if item_iter.peek().is_some() {
			    drw.w
			} else {
			    drw.w - rangle_width
			}
		    }{  // not enough room, create new partition
			partitions.push(partition);
			partition = Vec::new();
			x = drw.pseudo_globals.promptw + drw.pseudo_globals.inputw
			    + langle_width + item.width;
		    }
		    partition.push(item);
		}
		if partition.len() > 0 { // grab any extras from the last page
		    partitions.push(partition);
		}
		Ok(partitions)
	    },
	    Vertical => {
		Ok(input.chunks(drw.config.lines as usize)
		   .map(|p| p.into_iter().map(|el| el.clone()).collect())
		   .collect())
	    },
	}
    }
    pub fn gen_matches(drw: &mut Drw) -> Result<Vec<Item>, String> {
	let re = match RegexBuilder::new(&regex::escape(&drw.input))
	    .case_insensitive(!drw.config.case_sensitive)
	    .build() {
		Ok(re) => re,
		Err(_) => return Err(format!("Could not build regex")),
	    };
	let mut exact:     Vec<Item> = Vec::new();
	let mut prefix:    Vec<Item> = Vec::new();
	let mut substring: Vec<Item> = Vec::new();
	for item in &drw.items.as_mut().unwrap().data {
	    match item.matches(&re) {
		MatchCode::Exact => exact.push(item.clone()),
		MatchCode::Prefix => prefix.push(item.clone()),
		MatchCode::Substring => substring.push(item.clone()),
		MatchCode::None => {}
	    }
	}
	exact.reserve(prefix.len()+substring.len());
	for item in prefix { // extend is broken for pointers
	    exact.push(item);
	}
	for item in substring {
	    exact.push(item);
	}
	Ok(exact)
    }
}
