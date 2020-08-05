use overrider::*;

use crate::config::ConfigDefault;
use crate::drw::Drw;
use crate::item::Item;
use crate::result::*;
use crate::clapflags::CLAP_FLAGS;

use unicode_segmentation::UnicodeSegmentation;

#[override_flag(flag = maxlength)]
impl Drw {
    pub fn postprocess_matches(&mut self, current_matches: Vec<Item>) -> CompResult<Vec<Item>> {
	let max_length_str_option = CLAP_FLAGS.value_of( "maxlength" );

	match max_length_str_option
	{
		None => {
			Err(Die::Stderr("Please specificy max length".to_owned()))
		},
		Some( max_length_str ) => 
		{
			let max_length_result = max_length_str.parse::<usize>();
			match max_length_result
			{
				Err( _ ) => Err(Die::Stderr("Please specificy a positive integer for max length".to_owned())),
				Ok( 0 ) => Ok( current_matches ),
				Ok( max_length ) => {
					// >= in place of = in case someoen pastes stuff in
					// when there is a paste functionality.
					if self.input.graphemes(true).count() >= max_length
					{
						self.dispose( self.input.graphemes(true).take( max_length ).collect(), true )?;
						Err(Die::Stdout("".to_owned()))
					}
					else
					{
						Ok(current_matches)
					}
				}
			}
		}
	}
	}
}

#[override_flag(flag = maxlength)]
impl ConfigDefault {
    pub fn nostdin() -> bool {
	true
    }
}
