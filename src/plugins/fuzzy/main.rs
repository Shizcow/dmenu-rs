use overrider::*;

#[allow(unused_imports)]
use crate::clapflags::CLAP_FLAGS;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::drw::Drw;
use crate::item::Item;
use crate::result::*;

#[override_flag(flag = nofuzz, invert = true)]
impl Drw {
    pub fn gen_matches(&mut self) -> CompResult<Vec<Item>> {
	let searchterm = self.input.clone();
	let matcher: Box<dyn FuzzyMatcher> = Box::new(SkimMatcherV2::default());
	let mut items: Vec<(Item, i64)> = 
	    self.get_items().iter().map(|item| {
		(item.clone(),
		 if let Some(score) = matcher.fuzzy_match(&item.text, &searchterm) {
		     -score
		 } else {
		     1
		 })
	    }).collect();
	items.retain(|(_, score)| *score <= 0);
	items.sort_by_key(|(item, _)| item.text.len()); // this prioritizes exact matches
	items.sort_by_key(|(_, score)| *score);

	Ok(items.into_iter().map(|(item, _)| item).collect())
    }
}
