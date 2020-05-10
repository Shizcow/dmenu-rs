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
    pub fn draw(&self, drw: &mut Drw, mut x: c_int) -> Option<c_int> { // gets an apropriate vec of matches
	unsafe {
	    // clacoffsets
	    let n: c_int = 
		if drw.pseudo_globals.lines > 0 {
		    drw.pseudo_globals.lines as c_int*drw.pseudo_globals.bh
		} else {
		    drw.pseudo_globals.mw - (drw.pseudo_globals.promptw + drw.pseudo_globals.inputw + drw.textw(Some(&"<".to_string())) + drw.textw(Some(&">".to_string())))
		};
	    
	    let rangle = ">".to_string();

	    println!("items: {:?}, matches: {:?}", self.data, self.data_matches);

	    let mut ret = None;
	    for (index, item) in self.data_matches.iter().enumerate() {
		if index == self.curr {
		    drw.setscheme(drw.pseudo_globals.schemeset[SchemeSel as usize]);
		} else {
		    drw.setscheme(drw.pseudo_globals.schemeset[SchemeNorm as usize]);
		    //TODO: drw.setscheme(drw.pseudo_globals.schemeset[SchemeOut as usize]);
		}
		x = (**item).draw(x, 0, drw.textw(Some(&(**item).text)).min(drw.pseudo_globals.mw - x - drw.textw(Some(&rangle))), drw);
		if x >= drw.pseudo_globals.mw/2 {
		    break;
		}
	    }
	    ret
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
