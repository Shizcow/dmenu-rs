//use crate::globals::*;
use crate::config::*;
//use crate::fnt::*;
use crate::result::*;
use crate::init::*;
use crate::util::*;

use atty::Stream;

pub struct Menu {
    xkb_state: xkbcommon::xkb::State,
    conn: xcb::Connection,
    cr: Cairo,
    layout: pango::Layout,
    items: Vec<String>,

    w: u16,
    h: i32,
}

// TODO: can we get this from the window manager?
const FONT: &str = "mono 12";

impl Menu {
    pub fn new(/*pseudo_globals: PseudoGlobals, */config: Config) -> CompResult<Self> {
	// set up connection to X server
	let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
	// init xinerama -- used later
	init_xinerama(&conn);
	// create window -- height is calculated later
	let (screen, window, w) = create_xcb_window(&conn, screen_num);
	let dpi = get_dpi(&screen);
	// set up cairo
	// TODO shrink
	let cr = create_cairo_context(&conn, &screen, &window, w.into(), 100);
	// set up pango
	let layout = create_pango_layout(&cr, FONT, dpi);

	
	layout.set_text(" ");
	let (_, text_height) = layout.get_size();
	let h = text_height/pango::SCALE;

	xcb::configure_window(&conn, window, &[(xcb::CONFIG_WINDOW_HEIGHT as u16, h as u32)]);
	xcb::map_window(&conn, window);
	
	

	let (items, xkb_state) = if config.nostdin {
	    (Vec::new(), setup_xkb(&conn, window))
	} else {
	    if config.fast && atty::is(Stream::Stdin) {
		let xkb_state = setup_xkb(&conn, window);
		(readstdin()?, xkb_state)
	    } else {
		let tmp = readstdin()?; // ensure order
		(tmp, setup_xkb(&conn, window))
	    }
	};

	//ret.config.lines = ret.config.lines.min(ret.get_items().len() as u32);
	 

	let event = conn.wait_for_event();
        if let Some(event) = event {
            let r = event.response_type() & !0x80;
            match r {
                xcb::EXPOSE => {
		    return Ok(Self{xkb_state, conn, layout, cr, w, h, items});
		}
		_ => {}
	    }
	}
	Die::stderr("xcb could not spawn".to_owned())
    }
    pub fn draw(&self) -> CompResult<()> {
	let norm = [parse_color("#bbb"), parse_color("#222")];
	let sel  = [parse_color("#eee"), parse_color("#057")];

	// background
	self.cr.context.set_source_rgb(norm[1][0], norm[1][1], norm[1][2]);
        self.cr.context.paint();

	let mut x = 100.0;
	for (i, item) in self.items.iter().take(5).enumerate() {
	    self.layout.set_text(item);
	    let (mut text_width, mut text_height) = self.layout.get_size();
	    text_width /= pango::SCALE;
	    text_height /= pango::SCALE;

	    println!("{} {}", text_height, self.h);

	    if i == 0 {
		self.cr.context.set_source_rgb(sel[1][0], sel[1][1], sel[1][2]);
	    } else {
		self.cr.context.set_source_rgb(norm[1][0], norm[1][1], norm[1][2]);
	    }
            self.cr.context.rectangle(x, 0.0, text_width as f64 + 10.0, self.h.into());
            self.cr.context.fill();
	    
	    self.cr.context.set_source_rgb(norm[0][0], norm[0][1], norm[0][2]);
	    self.cr.context.move_to(x + 5.0,0.0);
	    pangocairo::show_layout(&self.cr.context, &self.layout);

	    x += text_width as f64 + 10.0; // TODO: lrpad
	}

	/*
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
	self.cr.context.set_source_rgb(1.0, 1.0, 1.0);
	// place and draw text
	self.cr.context.move_to((self.w as i32 -text_width) as f64/2.0,
			(self.h as i32-text_height) as f64/2.0);
	pangocairo::show_layout(&self.cr.context, &self.layout);
	 */

	// wait for everything to finish drawing before moving on
	self.conn.flush();
	Ok(())
    }
}


fn parse_color(s: &str) -> [f64; 3] {
    let c: css_color_parser::Color = s.parse().unwrap();
    [(c.r as f64)/255.0, (c.g as f64)/255.0, (c.b as f64)/255.0]
}
