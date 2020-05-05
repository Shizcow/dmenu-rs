use libc::c_int;

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
    pub fn matches(&self, text: &String) -> MatchCode {
	match self.text.match_indices(text).nth(0) {
	    None => MatchCode::None,
	    Some((0,_)) => if text.len() == self.text.len() {MatchCode::Exact} else {MatchCode::Substring},
	    Some(_) => MatchCode::Substring,
	}
    }
    pub fn matches_vec<'a>(text: &String, vec: &'a Vec<Self>) -> Vec<&'a Self> {
	let mut exact     = Vec::new();
	let mut prefix    = Vec::new();
	let mut substring = Vec::new();
	for item in vec {
	    match item.matches(text) {
		MatchCode::Exact => exact.push(item),
		MatchCode::Prefix => prefix.push(item),
		MatchCode::Substring => substring.push(item),
		MatchCode::None => {}
	    }
	}
	exact.reserve(prefix.len()+substring.len());
	exact.extend(prefix);
	exact.extend(substring);
	exact
    }
}
