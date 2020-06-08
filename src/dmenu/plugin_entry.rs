#[allow(unused_imports)]
use crate::clapflags::CLAP_FLAGS;
use overrider::*;

use crate::drw::Drw;

#[default]
impl Drw {
    pub fn format_input(&self) -> String {
	self.input.clone()
    }
}
