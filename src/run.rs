/* run.rs
 *
 * Holds the run method for Drw,
 *   as well as keypress handling
 */

use x11::xlib::{XRaiseWindow, XmbLookupString, VisibilityUnobscured, VisibilityNotify,
		SelectionNotify, DestroyNotify, FocusIn, Expose, False, XInternAtom,
		XEvent, XKeyEvent, XFilterEvent, XNextEvent, KeySym, KeyPress,
		Mod1Mask, ControlMask, XLookupChars, XLookupKeySym, XLookupBoth};
use libc::{iscntrl, c_char};
use std::mem::MaybeUninit;
use clipboard::{ClipboardProvider, ClipboardContext};
use regex::Regex;

use crate::util::grabfocus;
use crate::drw::Drw;

#[allow(non_upper_case_globals)]
impl Drw {
    pub fn run(&mut self) {
	unsafe{
	    let utf8 = XInternAtom(self.dpy, "UTF8_STRING\0".as_ptr() as *mut c_char, False);
	    let mut ev: XEvent = MaybeUninit::uninit().assume_init();
	    while XNextEvent(self.dpy, &mut ev) == 0 {
		if XFilterEvent(&mut ev, self.pseudo_globals.win) != 0 {
		    continue;
		}

		match ev.type_ {
		    DestroyNotify => {
			if ev.destroy_window.window != self.pseudo_globals.win {
			    break;
			}
		    },
		    Expose => {
			if ev.expose.count == 0 {
			    self.map(self.pseudo_globals.win, 0, 0, self.w, self.h);
			}
		    },
		    FocusIn => {
			/* regrab focus from parent window */
			//if ev.xfocus.window != self.pseudo_globals.win { TODO
			grabfocus(self);
			//}
		    },
		    KeyPress => {
			if self.keypress(ev.key) {
			    break;
			}
		    },
		    SelectionNotify => {
			if ev.selection.property == utf8 {
			    self.paste();
			}
		    },
		    VisibilityNotify => {
			if ev.visibility.state != VisibilityUnobscured {
			    XRaiseWindow(self.dpy, self.pseudo_globals.win);
			}
		    },
		    _ => {},
		}
	    }
	}
    }
    
    fn keypress(&mut self, mut ev: XKeyEvent) -> bool {
	use x11::keysym::*;
	unsafe {
	    let buf: [u8; 32] = [0; 32];
	    let mut __ksym: KeySym = MaybeUninit::uninit().assume_init();
	    let mut status = MaybeUninit::uninit().assume_init();
	    let len = XmbLookupString(self.pseudo_globals.xic, &mut ev, buf.as_ptr() as *mut i8, buf.len() as i32, &mut __ksym, &mut status);
	    let mut ksym = __ksym as u32;
	    match status {
		XLookupChars => {
		    if iscntrl(*(buf.as_ptr() as *mut i32)) == 0 {
			self.keyprocess(ksym, buf, len);
		    }
		},
		XLookupKeySym | XLookupBoth => {},
		_ => return false, /* XLookupNone, XBufferOverflow */
	    }
	    const control: bool = true;
	    const mod1:    bool = false;
	    if (ev.state & ControlMask) != 0 || (ev.state & Mod1Mask) != 0 {
		match (ksym, (ev.state & ControlMask) != 0) {		    
		    (XK_a, control)
			| (XK_g, mod1) => ksym = XK_Home,
		    (XK_b, control) => ksym = XK_Left,
		    (XK_c, control) => ksym = XK_Escape,
		    (XK_d, control) => ksym = XK_Delete,
		    (XK_e, control)
			| (XK_G, mod1) => ksym = XK_End,
		    (XK_f, control) => ksym = XK_Right,
		    (XK_g, control)
			| (XK_bracketleft, control) => ksym = XK_Escape,
		    (XK_h, control) => ksym = XK_BackSpace,
		    (XK_i, control) => ksym = XK_Tab,
		    (XK_j, mod1) => ksym = XK_Next,
		    (XK_k, mod1) => ksym = XK_Prior,
		    (XK_n, control)
			| (XK_l, mod1) => ksym = XK_Down,
		    (XK_p, control)
			| (XK_h, mod1) => ksym = XK_Up,
		    (XK_j, control)
			| (XK_J, control)
			| (XK_m, control)
			| (XK_M, control) => {
			    ksym = XK_Return;
			    ev.state &= !ControlMask;
			},
		    (XK_k, control) => { // delete all to the left
			self.input = self.input.chars().take(self.pseudo_globals.cursor).collect();
			self.draw();
			return false;
		    },
		    (XK_u, control) => { // delete all to the right
			self.input = self.input.chars().skip(self.pseudo_globals.cursor).collect();
			self.pseudo_globals.cursor = 0;
			self.draw();
			return false;
		    },
		    (XK_w, control)
			| (XK_BackSpace, control) => { // Delete word to the left
			    let mut state = 0;
			    let mut found = 0;
			    self.input = self.input.char_indices().rev().filter_map(|(i, c)|{
				if state == 0 && i < self.pseudo_globals.cursor {
				    state = 1; // searching for cursor
				}
				if state == 1 && c != ' ' {
				    state = 2; // looking for previous word
				}
				if state == 2 && c == ' ' {
				    state = 3; // skipping past next word
				}
				if state == 0 || state == 4 {
				    Some(c)
				} else if state == 3 {
				    found = i+1;
				    state = 4;
				    Some(c)
				} else {
				    None
				}
			    }).collect::<Vec<char>>().into_iter().rev().collect();
			    self.pseudo_globals.cursor = found;
			    self.draw();
			    return false;
			},
		    (XK_Delete, control) => { // Delete word to the right
			let mut state = 0;
			self.input = self.input.char_indices().filter_map(|(i, c)|{
			    if state == 0 && i >= self.pseudo_globals.cursor {
				state = 1; // searching for cursor
			    }
			    if state == 1 && c != ' ' {
				state = 2; // looking for next word
			    }
			    if state == 2 && c == ' ' {
				state = 3; // skipping past next word
			    }
			    if state == 0 || state == 4 {
				Some(c)
			    } else if state == 3 {
				state = 4;
				Some(c)
			    } else {
				None
			    }
			}).collect();
			self.draw();
			return false;
		    }
		    (XK_y, control)
			| (XK_Y, control) => { // paste selection
			    self.paste();
			    return false;
			},
		    (XK_Left, control)
			| (XK_b, mod1) => { // skip to word boundary on left
			    self.pseudo_globals.cursor = 
				self.input.char_indices().rev()
				.skip(self.input.len()-self.pseudo_globals.cursor)
				.skip_while(|(_, c)| *c == ' ') // find last word
				.skip_while(|(_, c)| *c != ' ') // skip past it
				.next().map(|(i, _)| i+1)
				.unwrap_or(0);
			    self.draw();
			    return false;
			},
		    (XK_Right, control)
			| (XK_f, mod1) => { // skip to word boundary on right
			    self.pseudo_globals.cursor = 
				self.input.char_indices().skip(self.pseudo_globals.cursor+1)
				.skip_while(|(_, c)| *c == ' ') // find next word
				.skip_while(|(_, c)| *c != ' ') // skip past it
				.next().map(|(i, _)| i)
				.unwrap_or(self.input.len());
			    self.draw();
			    return false;
			},
		    (XK_Return, control)
			| (XK_KP_Enter, control) => {},
		    _ => return false,
		}
	    }
	    self.keyprocess(ksym, buf, len)
	}
    }
    
    fn keyprocess(&mut self, ksym: u32, buf: [u8; 32], len: i32) -> bool {
	use x11::keysym::*;
	unsafe {
	    match ksym {
		XK_Escape => return true,
		XK_Return | XK_KP_Enter => {
		    if self.items.data_matches.len() > 0 { // find the current selection
			let (partition_i, partition) = {
			    let mut partition_i = self.items.curr;
			    let mut partition = 0;
			    for p in &self.items.data_matches {
				if partition_i >= p.len() {
				    partition_i -= p.len();
				    partition += 1;
				} else {
				    break;
				}
			    }
			    (partition_i, partition)
			};
			// and print
			println!("{}", (*self.items.data_matches[partition][partition_i]).text);
		    }
		    return true;
		},
		XK_Tab => {
		    if self.items.data_matches.len() > 0 { // find the current selection
			let (partition_i, partition) = {
			    let mut partition_i = self.items.curr;
			    let mut partition = 0;
			    for p in &self.items.data_matches {
				if partition_i >= p.len() {
				    partition_i -= p.len();
				    partition += 1;
				} else {
				    break;
				}
			    }
			    (partition_i, partition)
			}; // and autocomplete
			self.input = (*self.items.data_matches[partition][partition_i]).text.clone();
			self.pseudo_globals.cursor = self.input.len();			
			self.items.curr = 0;
			self.draw();
		    }
		},
		XK_Home => {
		    if self.items.data_matches.len() > 0 {
			self.items.curr = 0;
			self.draw();
		    }
		},
		XK_End => {
		    if self.items.data_matches.len() > 0 {
			self.items.curr = self.items.data_matches.iter().fold(0, |acc, cur| acc+cur.len())-1;
			self.draw();
		    }
		},
		XK_Next => { // PgDn
		    let mut partition_i = self.items.curr;
		    let mut partition = 0;
		    for p in &self.items.data_matches {
			if partition_i >= p.len() {
			    partition_i -= p.len();
			    partition += 1;
			} else {
			    break;
			}
		    }
		    if partition+1 < self.items.data_matches.len() {
			self.items.curr += self.items.data_matches[partition].len()-partition_i;
			self.draw();
		    }
		},
		XK_Prior => { // PgUp
		    let mut partition_i = self.items.curr;
		    let mut partition = 0;
		    for p in &self.items.data_matches {
			if partition_i >= p.len() {
			    partition_i -= p.len();
			    partition += 1;
			} else {
			    break;
			}
		    }
		    if partition > 0 {
			self.items.curr -= self.items.data_matches[partition-1].len()+partition_i;
			self.draw();
		    }
		},
		XK_Left => {
		    if self.config.lines == 0 && self.pseudo_globals.cursor == self.input.len() && self.items.curr > 0 { // move selection
			self.items.curr -= 1;
			self.draw();
		    } else { // move cursor
			if self.pseudo_globals.cursor > 0 {
			    self.pseudo_globals.cursor -= 1;
			    self.draw();
			}
		    }
		},
		XK_Right => {
		    if self.config.lines == 0 && self.pseudo_globals.cursor == self.input.len() { // move selection
			if self.items.curr+1 < self.items.data_matches.iter().fold(0, |acc, cur| acc+cur.len()) {
			    self.items.curr += 1;
			    self.draw();
			}
		    } else { // move cursor
			if self.pseudo_globals.cursor < self.input.len() {
			    self.pseudo_globals.cursor += 1;
			    self.draw();
			}
		    }
		},
		XK_Up => {
		    if self.items.curr > 0 {
			self.items.curr -= 1;
			self.draw();
		    }
		},
		XK_Down => {
		    if self.items.curr+1 < self.items.data_matches.iter().fold(0, |acc, cur| acc+cur.len()) {
			self.items.curr += 1;
			self.draw();
		    }
		},
		XK_BackSpace => {
		    if self.pseudo_globals.cursor > 0 {
			let mut iter = self.input.drain(..).collect::<Vec<char>>().into_iter();
			self.input.push_str(&(&mut iter).take(self.pseudo_globals.cursor-1).collect::<String>());
			iter.next(); // get rid of one char
			self.input.push_str(&iter.collect::<String>());
			self.pseudo_globals.cursor -= 1;
			self.draw();
		    }
		},
		XK_Delete => {
		    if self.pseudo_globals.cursor < self.input.len() {
			let mut iter = self.input.drain(..).collect::<Vec<char>>().into_iter();
			self.input.push_str(&(&mut iter).take(self.pseudo_globals.cursor).collect::<String>());
			iter.next(); // get rid of one char
			self.input.push_str(&iter.collect::<String>());
			self.draw();
		    }
		},
		_ => { // all others, assumed to be normal chars
		    if iscntrl(*(buf.as_ptr() as *mut i32)) == 0 {
			let mut iter = self.input.drain(..).collect::<Vec<char>>().into_iter();
			self.input = (&mut iter).take(self.pseudo_globals.cursor).collect();
			self.pseudo_globals.cursor += buf[..len as usize].iter()
			    .fold(0, |acc, c| acc + if *c > 0 {1} else {0});
			self.input.push_str(&String::from_utf8_lossy(&buf[..len as usize]));
			self.input.push_str(&iter.collect::<String>());
			self.items.curr = 0;
			self.draw();
		    }
		},
	    }
	}
	false
    }

    fn paste(&mut self) { // paste selection and redraw
	let mut ctx: ClipboardContext = ClipboardProvider::new()
	    .expect("Could not grab clipboard");
	if let Ok(mut clip) = ctx.get_contents() {
	    clip = Regex::new(r"[\t]").expect("Cannot build regex")
		.replace_all(&Regex::new(r"[\r\n]").expect("Cannot build regex")
			     .replace_all(&clip, "").to_string() // remove newlines
			     , "    ").to_string(); // replace tab with 4 spaces
	    let mut iter = self.input.drain(..).collect::<Vec<char>>().into_iter();
	    self.input = (&mut iter).take(self.pseudo_globals.cursor).collect();
	    self.input.push_str(&clip);
	    self.input.push_str(&iter.collect::<String>());
	    self.pseudo_globals.cursor += clip.len();
	    self.draw();
	}
    }
}
