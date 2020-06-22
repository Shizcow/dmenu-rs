use overrider::*;

use crate::drw::Drw;
use crate::item::Item;

use rink::*;


#[override_flag(flag = calc)]
impl Drw {
    pub fn gen_matches(&mut self) -> Result<Vec<Item>, String> {
	
	let mut ctx = load().unwrap();
	if let Ok(evaluated) = one_line(&mut ctx, &self.input) {
	    Ok(vec![match Item::new(evaluated, false, self) {
		Ok(item) => item,
		Err(err) => return Err(err),
	    }])
	} else {
	    Ok(vec![])
	}
    }
}
