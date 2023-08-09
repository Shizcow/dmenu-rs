use crate::config::{DefaultWidth, Schemes::*};
use crate::drw::{Drw, TextOption::*};
use crate::result::*;

use libc::c_int;
use regex::Regex;

#[allow(unused_imports)]
pub enum MatchCode {
    Exact,
    Prefix,
    Substring,
    None,
}
pub use MatchCode::*;
#[derive(Debug)]
pub enum Direction {
    Vertical,
    Horizontal,
}
pub use Direction::*;

#[derive(Debug, Clone)]
pub struct Item {
    // dmenu entry
    pub text: String,
    pub out: bool,
    pub width: c_int,
}

impl Item {
    pub fn new(text: String, out: bool, drw: &mut Drw) -> CompResult<Self> {
        Ok(Self {
            out,
            width: drw.textw(Other(&text))?,
            text,
        })
    }
    pub fn draw(&self, x: c_int, y: c_int, w: c_int, drw: &mut Drw) -> CompResult<c_int> {
        drw.text(
            x,
            y,
            w as u32,
            drw.pseudo_globals.bh as u32,
            drw.pseudo_globals.lrpad as u32 / 2,
            Other(&self.text),
            false,
        )
        .map(|o| o.0)
    }
    #[allow(unused)] // won't be used if overriden
    pub fn matches(&self, re: &Regex) -> MatchCode {
        match re
            .find_iter(&self.text)
            .nth(0)
            .map(|m| (m.start(), m.end()))
            .unwrap_or((1, 0))
        {
            (1, 0) => MatchCode::None, // don't expect zero length matches...
            (0, end) =>
            //                unless search is empty
            {
                if end == self.text.len() {
                    MatchCode::Exact
                } else {
                    MatchCode::Prefix
                }
            }
            _ => MatchCode::Substring,
        }
    }
}

#[derive(Debug)]
pub struct Partition {
    pub data: Vec<Item>,
    pub leftover: i32, // leftover padding on right side
}

impl Partition {
    pub fn new(data: Vec<Item>, leftover: i32) -> Self {
        Self { data, leftover }
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn decompose(haystack: &Vec<Self>, needle: &Drw) -> (usize, usize) {
        let mut partition_i = needle.items.as_ref().unwrap().curr;
        let mut partition = 0;
        for p in haystack {
            if partition_i >= p.len() {
                partition_i -= p.len();
                partition += 1;
            } else {
                break;
            }
        }
        (partition_i, partition)
    }
}

impl std::ops::Index<usize> for Partition {
    type Output = Item;

    fn index(&self, index: usize) -> &Item {
        &self.data[index]
    }
}

#[derive(Debug)]
pub struct Items {
    pub data: Vec<Item>,
    pub cached_partitions: Vec<Partition>, // seperated into screens
    pub curr: usize,
}

impl Items {
    pub fn new(data: Vec<Item>) -> Self {
        Self {
            data,
            cached_partitions: Vec::new(),
            curr: 0,
        }
    }
    pub fn match_len(&self) -> usize {
        self.cached_partitions.len()
    }
    pub fn draw(drw: &mut Drw, direction: Direction) -> CompResult<bool> {
        // gets an apropriate vec of matches
        let pre_processed_items = drw.gen_matches()?;
        let items_to_draw = drw.postprocess_matches(pre_processed_items)?;
        let rangle = ">".to_string();
        let rangle_width = drw.textw(Other(&rangle))?;
        let langle = "<".to_string();
        let langle_width = drw.textw(Other(&langle))?;

        drw.pseudo_globals.inputw = match drw.config.render_default_width {
            DefaultWidth::Min => items_to_draw
                .iter()
                .fold(0, |acc, w| acc.max(w.width))
                .min(drw.w / 3)
                .min(drw.textw(Input)?),
            DefaultWidth::Items => drw
                .get_items()
                .iter()
                .fold(0, |acc, w| acc.max(w.width))
                .min(drw.w / 3),
            DefaultWidth::Max => {
                let curr = drw.items.as_ref().unwrap().curr;
                let data = drw.get_items();
                let mut w = drw.w - drw.pseudo_globals.promptw - data[curr].width;
                if curr < data.len() - 1 {
                    w -= rangle_width;
                }
                if curr > 0 {
                    w -= langle_width;
                }
                w
            }
            DefaultWidth::Custom(width) => (drw.w as f32 * (width as f32) / 100.0) as i32,
        };

        let matched_partitions = Self::partition_matches(
            items_to_draw,
            &direction,
            drw,
            if !(drw.config.render_default_width == DefaultWidth::Min)
                || drw.config.render_default_width == DefaultWidth::Items
            {
                langle_width
            } else {
                0
            },
            rangle_width,
        )?;

        if matched_partitions.len() == 0 {
            drw.items.as_mut().unwrap().cached_partitions = matched_partitions;
            return Ok(false); // nothing to draw
        }

        let (partition_i, partition) = Partition::decompose(&matched_partitions, drw);

        let mut coord = match direction {
            Horizontal => {
                if drw.config.render_rightalign {
                    matched_partitions[partition].leftover
                } else {
                    0
                }
            }
            Vertical => drw.pseudo_globals.bh as c_int,
        };

        if let Horizontal = direction {
            if drw.config.render_flex {
                let inputw_desired = drw.textw(Input)?;
                if inputw_desired > drw.pseudo_globals.inputw {
                    let delta = inputw_desired
                        - drw.pseudo_globals.inputw
                        - matched_partitions[partition].leftover;
                    if delta < 0 {
                        drw.pseudo_globals.inputw = inputw_desired;
                    } else {
                        drw.pseudo_globals.inputw = inputw_desired - delta;
                    }
                }
            }
            coord += drw.pseudo_globals.promptw + drw.pseudo_globals.inputw;
            if partition > 0 {
                // draw langle if required
                drw.setscheme(SchemeNorm);
                coord = drw
                    .text(
                        coord,
                        0,
                        langle_width as u32,
                        drw.pseudo_globals.bh as u32,
                        drw.pseudo_globals.lrpad as u32 / 2,
                        Other(&langle),
                        false,
                    )?
                    .0;
                if drw.config.render_default_width == DefaultWidth::Max {
                    // This is here due do an optical illusion
                    // It's not pedantically correct alignment, but makes sense on Max
                    drw.pseudo_globals.inputw += drw.pseudo_globals.lrpad / 2;
                }
            } else {
                // now, do we give phantom space?
                if drw.config.render_default_width == DefaultWidth::Items {
                    coord += langle_width;
                }
            }
        }

        for index in 0..matched_partitions[partition].len() {
            if index == partition_i {
                drw.setscheme(SchemeSel);
            } else if matched_partitions[partition][index].out {
                drw.setscheme(SchemeOut);
            } else {
                drw.setscheme(SchemeNorm);
            }
            match direction {
                Horizontal => {
                    if partition + 1 < matched_partitions.len() {
                        // draw rangle
                        coord = matched_partitions[partition][index].draw(
                            coord,
                            0,
                            matched_partitions[partition][index]
                                .width
                                .min(drw.w - coord - rangle_width),
                            drw,
                        )?;
                        drw.setscheme(SchemeNorm);
                        drw.text(
                            drw.w - rangle_width,
                            0,
                            rangle_width as u32,
                            drw.pseudo_globals.bh as u32,
                            drw.pseudo_globals.lrpad as u32 / 2,
                            Other(&rangle),
                            false,
                        )?;
                    } else {
                        // no rangle
                        coord = matched_partitions[partition][index].draw(
                            coord,
                            0,
                            matched_partitions[partition][index]
                                .width
                                .min(drw.w - coord),
                            drw,
                        )?;
                    }
                }
                Vertical => {
                    matched_partitions[partition][index].draw(0, coord, drw.w, drw)?;
                    coord += drw.pseudo_globals.bh as i32;
                }
            }
        }

        drw.items.as_mut().unwrap().cached_partitions = matched_partitions;

        Ok(true)
    }

    fn partition_matches(
        input: Vec<Item>,
        direction: &Direction,
        drw: &mut Drw,
        langle_width: i32,
        rangle_width: i32,
    ) -> CompResult<Vec<Partition>> {
        // matches come in, partitions come out
        match direction {
            Horizontal => {
                let mut partitions = Vec::new();
                let mut partition_build = Vec::new();
                let mut x = if drw.config.render_default_width == DefaultWidth::Items {
                    drw.pseudo_globals.promptw + drw.pseudo_globals.inputw + langle_width
                } else {
                    drw.pseudo_globals.promptw + drw.pseudo_globals.inputw
                };
                let mut item_iter = input.into_iter().peekable();
                while let Some(item) = item_iter.next() {
                    let precomp_width = x;
                    let leftover;
                    x += item.width;
                    if x > {
                        let width_comp = if item_iter.peek().is_some() {
                            drw.w - rangle_width
                        } else {
                            drw.w
                        };
                        leftover = width_comp - precomp_width;
                        width_comp
                    } || drw.config.render_default_width == DefaultWidth::Max
                    {
                        // not enough room, create new partition, but what if:
                        if !(
                            partitions.len() == 0           // if there's only one page
			     && item_iter.peek().is_none()   // there will only be one page
			     && x < drw.w + rangle_width
                            // and everything could fit if it wasn't for the '>'
                        ) && partition_build.len() > 0
                        {
                            // (make sure no empties)
                            partitions.push(Partition::new(partition_build, leftover));
                            partition_build = Vec::new();
                            x = drw.pseudo_globals.promptw
                                + drw.pseudo_globals.inputw
                                + langle_width
                                + item.width;
                        }
                    }
                    partition_build.push(item);
                }
                if partition_build.len() > 0 {
                    // grab any extras from the last page
                    let leftover = if partitions.len() == 0 {
                        drw.w - x
                    } else {
                        drw.w - x - langle_width
                    };
                    partitions.push(Partition::new(partition_build, leftover));
                }
                Ok(partitions)
            }
            Vertical => Ok(input
                .chunks(drw.config.lines as usize)
                .map(|p| Partition::new(p.to_vec(), 0))
                .collect()),
        }
    }
}

impl Drw {
    #[inline(always)]
    pub fn get_items(&self) -> &Vec<Item> {
        &self.items.as_ref().unwrap().data
    }
    #[allow(unused)] // for plugins
    #[inline(always)]
    pub fn get_items_mut(&mut self) -> &mut Vec<Item> {
        &mut self.items.as_mut().unwrap().data
    }
}
