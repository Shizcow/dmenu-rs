//use crate::item::{Items, Direction::*};
//use crate::globals::*;
//use crate::config::*;
//use crate::fnt::*;
use crate::result::*;
use crate::init::*;

pub struct Drw {
    xkb_state: xkbcommon::xkb::State,
    conn: xcb::Connection,
    cr: cairo::Context,
    layout: pango::Layout,

    w: u16,
    h: u16,
}

// TODO: automate
const FONT:  &str = "Office Code Pro 30";
const HEIGHT: u16 = 180;

impl Drw {
    pub fn new(/*pseudo_globals: PseudoGlobals, config: Config*/) -> CompResult<Self> {
	// get a size hint for menu height
	let font = pango::FontDescription::from_string(FONT);
	let text_height = font.get_size() / pango::SCALE;
	// set up connection to X server
	let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
	// init xinerama -- used later
	init_xinerama(&conn);
	// create window -- height is calculated later
	let (screen, window, (w, h)) = create_xcb_window(&conn, screen_num, ((text_height as f32) * 1.5) as u16);
	// grab keyboard
	let xkb_state = setup_xkb(&conn, window);
	// set up cairo
	let cr = create_cairo_context(&conn, &screen, &window, w.into(), h.into());
	// set up pango
	let layout = create_pango_layout(&cr, FONT);
	
	/*ret.items = if ret.config.nostdin {
	grabkeyboard(ret.dpy, ret.config.embed)?;
	Some(Items::new(Vec::new()))
    } else {Some(Items::new(
	if ret.config.fast && isatty(0) == 0 {
	grabkeyboard(ret.dpy, ret.config.embed)?;
	readstdin(&mut ret)?
    } else {
	let tmp = readstdin(&mut ret)?;
	grabkeyboard(ret.dpy, ret.config.embed)?;
	tmp
    }))
    };

	ret.config.lines = ret.config.lines.min(ret.get_items().len() as u32);
	 */

	let event = conn.wait_for_event();
        if let Some(event) = event {
            let r = event.response_type() & !0x80;
            match r {
                xcb::EXPOSE => {
		    return Ok(Self{xkb_state, conn, layout, cr, w, h});
		}
		_ => {}
	    }
	}
	Die::stderr("xcb could not spawn".to_owned())
    }
    pub fn draw(&self) -> CompResult<()> {
	self.cr.set_source_rgb(0.5, 0.5, 0.5);
        self.cr.paint();

	// red triangle
        self.cr.set_source_rgb(1.0, 0.0, 0.0);
        self.cr.move_to(0.0, 0.0);
        self.cr.line_to(self.w.into(), 0.0);
        self.cr.line_to(self.w.into(), self.h.into());
        self.cr.close_path();
        self.cr.fill();

	// blue center line
        self.cr.set_source_rgb(0.0, 0.0, 1.0);
        self.cr.set_line_width(20.0);
        self.cr.move_to(0.0, 0.0);
        self.cr.line_to(self.w.into(), self.h.into());
        self.cr.stroke();

	// get ready to draw text
	self.layout.set_text("hello world");
	// get a size hint for allignment
	let (mut text_width, mut text_height) = self.layout.get_size();
	// If the text is too wide, ellipsize to fit
	if text_width > self.w as i32*pango::SCALE {
	    text_width = self.w as i32*pango::SCALE;
	    self.layout.set_ellipsize(pango::EllipsizeMode::End);
	    self.layout.set_width(text_width);
	}
	// scale back -- pango doesn't uses large integers instead of floats
	text_width /= pango::SCALE;
	text_height /= pango::SCALE;
	// base text color (does not apply to color bitmap chars)
	self.cr.set_source_rgb(1.0, 1.0, 1.0);
	// place and draw text
	self.cr.move_to((self.w as i32 -text_width) as f64/2.0,
			(self.h as i32-text_height) as f64/2.0);
	pangocairo::show_layout(&self.cr, &self.layout);

	// wait for everything to finish drawing before moving on
	self.conn.flush();
	Ok(())
    }
}
