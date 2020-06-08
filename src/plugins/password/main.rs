//include!(concat!(env!("OUT_DIR"), "/proc_use.rs"));

use crate::drw::Drw;

use overrider::*;

#[override_flag(flag = password)]
impl Drw {
    pub fn format_input(&self) -> String {
	(0..self.input.len()).map(|_| "*").collect()
    }
}
