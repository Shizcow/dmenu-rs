use libc::c_int;
use crate::drw::Drw;
use crate::config::Schemes::*;

pub enum MatchCode {Exact, Prefix, Substring, None}

#[derive(Debug)]
pub struct Item { // dmenu entry
    pub text: String,
    pub out: c_int,
}

impl Item {
    pub fn new(text: String, out: c_int) -> Self {
	Self{text, out}
    }
    pub fn draw(&self, x: c_int, y: c_int, w: c_int, drw: &mut Drw) -> c_int {
	drw.text(x, y, w as u32, drw.pseudo_globals.bh as u32, drw.pseudo_globals.lrpad as u32/2, Some(&self.text), false)
    }
    pub fn matches(&self, text: &String) -> MatchCode {
	match self.text.match_indices(text).nth(0) {
	    None => MatchCode::None,
	    Some((0,_)) => if text.len() == self.text.len() {MatchCode::Exact} else {MatchCode::Substring},
	    Some(_) => MatchCode::Substring,
	}
    }
}

pub struct Items {
    data: Vec<Item>,
    data_matches: Vec<*const Item>, // TODO: can this be done with lifetimes?
    pub curr: usize,
}

impl Items {
    pub fn new(data: Vec<Item>) -> Self {
	Self{data, data_matches: Vec::new(), curr: 0}
    }
    pub fn match_len(&self) -> usize {
	self.data_matches.len()
    }
    pub fn draw(&self, drw: &mut Drw, mut x: c_int) { // gets an apropriate vec of matches
	unsafe {
	    
	    let rangle = ">".to_string();
	    let rangle_width =  drw.textw(Some(&rangle));

	    println!("items: {:?}, matches: {:?}", self.data, self.data_matches);

	    for index in 0..self.data_matches.len() {
		if index == self.curr {
		    drw.setscheme(drw.pseudo_globals.schemeset[SchemeSel as usize]);
		} else {
		    drw.setscheme(drw.pseudo_globals.schemeset[SchemeNorm as usize]);
		    //TODO: drw.setscheme(drw.pseudo_globals.schemeset[SchemeOut as usize]);
		}
		x = (*self.data_matches[index]).draw(x, 0, drw.textw(Some(&(*self.data_matches[index]).text)).min(drw.pseudo_globals.mw - x - rangle_width), drw);
		if index < self.data_matches.len()-2 { // Are there more items to draw
		    if x >= drw.pseudo_globals.mw - drw.textw(Some(&(*self.data_matches[index+1]).text)) - rangle_width { // check if they fit
			drw.setscheme(drw.pseudo_globals.schemeset[SchemeNorm as usize]); // TODO: optimize out multiple scheme switches
			// if not, draw >
			drw.text(drw.pseudo_globals.mw - rangle_width, 0, rangle_width as u32, drw.pseudo_globals.bh as u32, drw.pseudo_globals.lrpad as u32/2, Some(&rangle), false);
			break;
		    }
		}
	    }
	}
    }
    pub fn gen_matches(&mut self, text: &String) { // TODO: merge into draw?
	self.curr = 0;
	self.data_matches.clear();
	let mut prefix    = Vec::new();
	let mut substring = Vec::new();
	for item in &self.data {
	    match item.matches(text) {
		MatchCode::Exact => self.data_matches.push(item),
		MatchCode::Prefix => prefix.push(item),
		MatchCode::Substring => substring.push(item),
		MatchCode::None => {}
	    }
	}
	self.data_matches.reserve(prefix.len()+substring.len());
	for item in prefix {
	    self.data_matches.push(item);
	}
	for item in substring {
	    self.data_matches.push(item);
	}
    }
}
