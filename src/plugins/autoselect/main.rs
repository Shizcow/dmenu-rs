use overrider::*;

use crate::drw::Drw;
use crate::item::Item;
use crate::result::*;

#[override_flag(flag = autoselect)]
impl Drw {
    pub fn postprocess_matches(&mut self, mut current_matches: Vec<Item>) -> CompResult<Vec<Item>> {
	if current_matches.len() == 1 {
	    self.dispose(current_matches.swap_remove(0).text, true)?;
	    Err(Die::Stdout("".to_owned()))
	} else {
	    Ok(current_matches)
	}
    }
}
