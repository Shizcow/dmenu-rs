use overrider::*;

use crate::drw::Drw;
use crate::item::Item;
use crate::result::*;


#[override_flag( flag=autoselect )]
impl Drw {
    pub fn postprocess_matches( &mut self, current_matches: Vec<Item> ) -> CompResult<Vec<Item>> {
	if	
	    current_matches.len() == 1 
	{
	    Err( Die::Stdout( current_matches[ 0 ].text.to_string() ) )
	}
	else
	{
	    Ok( current_matches )
	}
    }
}
