use overrider::*;

use crate::drw::Drw;
use crate::result::*;

#[override_flag(flag = password)]
impl Drw {
    pub fn format_input(&self) -> CompResult<String> {
	Ok((0..self.input.len()).map(|_| "*").collect())
    }
}
