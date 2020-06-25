use overrider::*;
use rink::*;

use crate::drw::Drw;
use crate::item::Item;

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

use crate::config::InputFlex;
use crate::config::ConfigDefault;
#[override_flag(flag = calc)]
impl ConfigDefault {
    pub fn input_flex() -> InputFlex {
	InputFlex::Flex
    }
}
