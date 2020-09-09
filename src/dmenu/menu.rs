//use crate::globals::*;
use crate::config::*;
//use crate::fnt::*;
use crate::result::*;
use crate::init::*;
use crate::util::*;

use atty::Stream;

pub struct XCB {
    conn: xcb::Connection,
    window: xcb::Window,
}

pub struct Menu {
    xkb_state: xkbcommon::xkb::State, // TODO: wrap this into struct
    xcb: XCB,
    cairo: Cairo,
    layout: pango::Layout,
    items: Vec<String>,

    w: i32, // TODO: logically these should be unsigned
    h: i32,
}

// TODO: can we get this from the window manager?
const FONT: &str = "mono 12";

impl Menu {
    pub fn resize(&mut self, width: Option<i32>, height: Option<i32>/*, screen: &xcb::Screen*/) {
	let w = width.unwrap_or(self.w);
	let h = height.unwrap_or(self.h);
	if h != self.h {
	    xcb::configure_window(&self.xcb.conn, self.xcb.window, &[(xcb::CONFIG_WINDOW_HEIGHT as u16, h as u32)]);
	}
	if w != self.w {
	    xcb::configure_window(&self.xcb.conn, self.xcb.window, &[(xcb::CONFIG_WINDOW_WIDTH as u16, w as u32)]);
	}
	self.cairo.resize(w, h);
    }
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
	let mut cr = Cairo::new(&conn, &screen, &window, w.into());
	// set up pango
	let layout = create_pango_layout(&cr, FONT, dpi);
	
	layout.set_text(" ");
	let (_, text_height) = layout.get_size();
	let h = text_height/pango::SCALE;

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
	if event.is_none() || event.unwrap().response_type() & !0x80 != xcb::EXPOSE {
	    return Die::stderr("xcb could not spawn".to_owned());
	}
	let mut menu = Self{xkb_state, xcb: XCB{conn, window}, layout, cairo: cr, w, h: 0, items};

	menu.resize(None, Some(h));

	Ok(menu)
    }
    pub fn draw(&self) -> CompResult<()> {
	let norm = [parse_color("#bbb"), parse_color("#222")];
	let sel  = [parse_color("#eee"), parse_color("#057")];

	let lrpad = 10.0;

	// background
	self.cairo.context.set_source_rgb(norm[1][0], norm[1][1], norm[1][2]);
        self.cairo.context.paint();

	// items
	let mut x = 150.0;
	for (i, item) in self.items.iter().take(5).enumerate() {
	    self.layout.set_text(item);
	    let (mut text_width, mut text_height) = self.layout.get_size();
	    text_width /= pango::SCALE;
	    text_height /= pango::SCALE;

	    if i == 0 {
		self.cairo.context.set_source_rgb(sel[1][0], sel[1][1], sel[1][2]);
	    } else {
		self.cairo.context.set_source_rgb(norm[1][0], norm[1][1], norm[1][2]);
	    }
            self.cairo.context.rectangle(x, 0.0, text_width as f64 + lrpad, self.h.into());
            self.cairo.context.fill();
	    
	    self.cairo.context.set_source_rgb(norm[0][0], norm[0][1], norm[0][2]);
	    self.cairo.context.move_to(x + lrpad/2.0,0.0);
	    pangocairo::show_layout(&self.cairo.context, &self.layout);

	    x += text_width as f64 + 10.0; // TODO: lrpad
	}
	

	// wait for everything to finish drawing before moving on
	self.xcb.conn.flush();
	Ok(())
    }

    pub fn watch_for_keystroke(&mut self) -> CompResult<()> {

	loop {
            let event = self.xcb.conn.wait_for_event();
            match event {
		None => {
                    break;
		},
		Some(event) => {
                    let r = event.response_type() & !0x80;
                    match r {
			xcb::KEY_PRESS => {
			    
			    // Use xkb to convert from key event to key name, the print
                            let key_press: &xcb::KeyPressEvent = unsafe{xcb::cast_event(&event)};
			    let keycode = key_press.detail().into();
			    
			    let sym = self.xkb_state.key_get_one_sym(keycode);

			    let name = xkbcommon::xkb::keysym_get_name(sym);

			    if self.xkb_state.mod_name_is_active(xkbcommon::xkb::MOD_NAME_CTRL, xkbcommon::xkb::STATE_MODS_EFFECTIVE) {
				print!("C-");
			    }
			    if self.xkb_state.mod_name_is_active(xkbcommon::xkb::MOD_NAME_ALT,  xkbcommon::xkb::STATE_MODS_EFFECTIVE) {
				print!("M-");
			    }
			    if self.xkb_state.mod_name_is_active(xkbcommon::xkb::MOD_NAME_LOGO, xkbcommon::xkb::STATE_MODS_EFFECTIVE) {
				print!("s-");
			    }
                            println!("{}", name);
			    if name == "q" {
				break;
			    }
			    
			    self.xkb_state.update_key(keycode, xkbcommon::xkb::KeyDirection::Down);
			},
			xcb::KEY_RELEASE => {
                            let key_press: &xcb::KeyPressEvent = unsafe{xcb::cast_event(&event)};
			    let keycode = key_press.detail().into();
			    self.xkb_state.update_key(keycode, xkbcommon::xkb::KeyDirection::Up);
			},
			xcb::MAP_NOTIFY => {
			    self.xkb_state = reload_xkb_map(&self.xcb.conn);
			},
			_ => {}
                    }
		}
            }
	}

	
	Ok(())
    }
}


fn parse_color(s: &str) -> [f64; 3] {
    let c: css_color_parser::Color = s.parse().unwrap();
    [(c.r as f64)/255.0, (c.g as f64)/255.0, (c.b as f64)/255.0]
}
