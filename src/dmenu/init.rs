use itertools::Itertools;

pub fn init_xinerama(conn: &xcb::Connection) {
    conn.prefetch_extension_data(xcb::xinerama::id());

    // generally useful to retrieve the first event from this
    // extension. event response_type will be set on this
    let _first_ev = match conn.get_extension_data(xcb::xinerama::id()) {
        Some(r) => r.first_event(),
        None => { panic!("Xinerama extension not supported by X server!"); }
    };

    // xinerama is very well supported, so version check isn't needed
}

/// Utility function: used when setting up xcb
pub fn get_root_visual_type(screen: &xcb::Screen) -> xcb::Visualtype {
    screen.allowed_depths()
	.flat_map(|depth| depth.visuals())
	.find(|visual| screen.root_visual() == visual.visual_id())
	.expect("No visual type found")
}

/// Create cairo context for drawing, links to xcb here
pub fn create_cairo_context(conn: &xcb::Connection,
                        screen: &xcb::Screen,
                        window: &xcb::Window,
			width: i32,
			height: i32)
                        -> cairo::Context {
    let surface;
    unsafe {
        let cairo_conn = cairo::XCBConnection::from_raw_none(conn.get_raw_conn() as
                                                             *mut cairo_sys::xcb_connection_t);
        let visual_ptr: *mut cairo_sys::xcb_visualtype_t =
            &mut get_root_visual_type(&screen).base as *mut _ as *mut cairo_sys::xcb_visualtype_t;
        let visual = cairo::XCBVisualType::from_raw_none(visual_ptr);
        let cairo_screen = cairo::XCBDrawable(window.to_owned());
        surface = cairo::XCBSurface::create(&cairo_conn, &cairo_screen, &visual, width, height).unwrap();
    }

    cairo::Context::new(&surface)
}

/// Create a pango layout, used for drawing text, links to cairo
pub fn create_pango_layout(cr: &cairo::Context, font: &str) -> pango::Layout {
    let layout = pangocairo::create_layout(&cr).unwrap();
    layout.set_font_description(Some(&pango::FontDescription::from_string(font)));
    layout
}

fn intersect_geo(window: &xcb::GetGeometryReply, screen: &xcb::xinerama::ScreenInfo) -> u32 {
    let (x1, y1) = (window.x()        as i32, window.y()           as i32);
    let (x2, y2) = (x1+window.width() as i32, y1+window.height() as i32);
    intersect(x1, x2, y1, y2, screen)
}

/// Takes a rectangle and a screen, calculates how many pixels they overlap on
fn intersect(x1: i32, x2: i32, y1: i32, y2: i32, screen: &xcb::xinerama::ScreenInfo) -> u32 {
    let (s_x1, s_y1) = (screen.x_org()      as i32, screen.y_org()       as i32);
    let (s_x2, s_y2) = (s_x1+screen.width() as i32, s_y1+screen.height() as i32);
    (0.max(x2.min(s_x2)-x1.max(s_x1)) * 0.max(y2.min(s_y2)-y1.max(s_y1))) as u32
}

fn parent_win(conn: &xcb::Connection, w: xcb::Window) -> xcb::Window {
    xcb::query_tree(conn, w)
	.get_reply().unwrap().parent()
}

/// Creates and initialized an xcb window, returns (screen, window)
pub fn create_xcb_window<'a>(conn: &'a xcb::Connection, screen_num: i32, x: i16, y: i16, width: u16, height: u16) -> (xcb::StructPtr<'a, xcb::ffi::xcb_screen_t>, u32) {

    // init connection to X server
    let screen =
	conn.get_setup().roots().nth(screen_num as usize).unwrap();

    // TODO: override with .mon command line option
    // TODO: support disabling xinerama

    // Xinerama: Where should the window be placed?
    let found_window = xcb::get_input_focus(&conn).get_reply().unwrap().focus();
    let mut focused_window = found_window;
    while {
	focused_window = parent_win(conn, focused_window); // do
	focused_window == screen.root()                    // while
    }{}

    let active_screen =
	if focused_window == 0 {
	    let pointer = xcb::query_pointer(&conn, screen.root()).get_reply().unwrap();

	    let (x1, y1) = (pointer.root_x() as i32, pointer.root_y() as i32);
	    xcb::xinerama::query_screens(&conn)
		.get_reply().unwrap().screen_info()
		.find(|screen| intersect(x1, x1+1,
					 y1, y1+1,
					 &screen) > 0).unwrap()
	} else {
	    let geometry = xcb::get_geometry(&conn, focused_window)
		.get_reply().unwrap();
	    
	    xcb::xinerama::query_screens(&conn)
		.get_reply().unwrap().screen_info()
		.sorted_by(|a, b| {
		    Ord::cmp(&intersect_geo(&geometry, &b), &intersect_geo(&geometry, &a))
		})
		.nth(0).unwrap()
	};
    
    

    let window = conn.generate_id();
    xcb::create_window(&conn,
		       xcb::COPY_FROM_PARENT as u8,
		       window,
		       screen.root(),
		       active_screen.x_org(), active_screen.y_org(),
		       width, height,
		       0,
		       xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
		       screen.root_visual(),
		       &[(xcb::CW_EVENT_MASK,
                          xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS),
                         (xcb::CW_OVERRIDE_REDIRECT, 1)
		       ]);

    xcb::map_window(&conn, window);

    (screen, window)
}

/// sets up xkb
pub fn setup_xkb(conn: &xcb::Connection, window: xcb::Window) -> xkbcommon::xkb::State {

    use xcb::xkb;
    
    conn.prefetch_extension_data(xkb::id());

    // generally useful to retrieve the first event from this
    // extension. event response_type will be set on this
    let _first_ev = match conn.get_extension_data(xkb::id()) {
        Some(r) => r.first_event(),
        None => { panic!("XKB extension not supported by X server!"); }
    };


    // we need at least xcb-xkb-1.0 to be available on client
    // machine
    let cookie = xkb::use_extension(&conn, 1, 0);

    match cookie.get_reply() {
        Ok(r) => {
            if !r.supported() {
                panic!("xkb-1.0 is not supported");
            }
        },
        Err(_) => {
            panic!("could not get xkb extension supported version");
        }
    };

    // we now select what events we want to receive
    // such as map change, keyboard hotplug ...
    // note that key strokes are given directly by
    // the XCB_KEY_PRESS event from xproto, not by xkb
    let map_parts =
        xkb::MAP_PART_KEY_TYPES |
    xkb::MAP_PART_KEY_SYMS |
    xkb::MAP_PART_MODIFIER_MAP |
    xkb::MAP_PART_EXPLICIT_COMPONENTS |
    xkb::MAP_PART_KEY_ACTIONS |
    xkb::MAP_PART_KEY_BEHAVIORS |
    xkb::MAP_PART_VIRTUAL_MODS |
    xkb::MAP_PART_VIRTUAL_MOD_MAP;

    let events =
        xkb::EVENT_TYPE_NEW_KEYBOARD_NOTIFY |
    xkb::EVENT_TYPE_MAP_NOTIFY |
    xkb::EVENT_TYPE_STATE_NOTIFY;

    let cookie = xkb::select_events_checked(&conn,
					    xkb::ID_USE_CORE_KBD as u16,
					    events as u16, 0, events as u16,
					    map_parts as u16, map_parts as u16, None);

    cookie.request_check().expect("failed to select notify events from xcb xkb");

    // grab keyboard -- needed because of redirect override
    let cookie = xcb::grab_keyboard(&conn, true, window, xcb::CURRENT_TIME, xcb::GRAB_MODE_ASYNC as u8, xcb::GRAB_MODE_ASYNC as u8);

    assert!(cookie.get_reply().expect("failed to get reply while grabbing keyboard").status() as u32
	    == xcb::GRAB_STATUS_SUCCESS, "failed to grab keyboard: invalid something");

    reload_xkb_map(conn)
}

/// utility function -- re/load the keymap
pub fn reload_xkb_map(conn: &xcb::Connection) -> xkbcommon::xkb::State {
    let context = xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS);
    let id = xkbcommon::xkb::x11::get_core_keyboard_device_id(conn);
    let keymap = xkbcommon::xkb::x11::keymap_new_from_device(&context, conn, id, xkbcommon::xkb::KEYMAP_COMPILE_NO_FLAGS);
    xkbcommon::xkb::x11::state_new_from_device(&keymap, conn, id)
}
