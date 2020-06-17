use overrider::*;

use crate::drw::Drw;
use crate::item::Item;

// deps: sagemath, python-babel

#[override_flag(flag = calc)]
impl Drw {
    pub fn gen_matches(&mut self) -> Result<Vec<Item>, String> {
	let out = match Item::new(self.input.clone(), false, self) {
	    Ok(item) => item,
	    Err(err) => return Err(err),
	};
	Ok(vec![out])
    }
}
