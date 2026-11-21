//! kitty's graphics protocol, through Unicode placeholders.
//!
//! One escape per picture asks kitty to read the PNG from its path (`t=f`,
//! `f=100`) and to create a *virtual* placement `c` x `r` cells big
//! (`U=1`), which kitty fits with the aspect ratio kept. The picture then
//! appears wherever cells hold `U+10EEEE` with a row and a column diacritic
//! and a foreground colour equal to the image id. Those cells live in the
//! frame's buffer like any text, so ratatui's diff redraws, moves and clears
//! them, and the pane clips them. Nothing is decoded here: kitty opens the
//! file, which is why this costs no image crate (docs/specs/25-image-pane.md).
//!
//! What goes: an id whose placement is no longer asked for is deleted with
//! `a=d,d=I` (the image and its data), and `clear` deletes every id this
//! backend allocated.

use std::io::{self, Write};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use tuipalette_core::ImageRequest;

use super::Graphics;

/// kitty's placeholder character.
const PLACEHOLDER: char = '\u{10EEEE}';

/// Row and column numbers, as kitty's `rowcolumn-diacritics.txt` lists
/// them: entry `n` encodes `n`. 297 entries, so a picture is at most 297
/// cells on a side.
pub const DIACRITICS: [char; 297] = [
    '\u{0305}', '\u{030D}', '\u{030E}', '\u{0310}', '\u{0312}', '\u{033D}', '\u{033E}', '\u{033F}',
    '\u{0346}', '\u{034A}', '\u{034B}', '\u{034C}', '\u{0350}', '\u{0351}', '\u{0352}', '\u{0357}',
    '\u{035B}', '\u{0363}', '\u{0364}', '\u{0365}', '\u{0366}', '\u{0367}', '\u{0368}', '\u{0369}',
    '\u{036A}', '\u{036B}', '\u{036C}', '\u{036D}', '\u{036E}', '\u{036F}', '\u{0483}', '\u{0484}',
    '\u{0485}', '\u{0486}', '\u{0487}', '\u{0592}', '\u{0593}', '\u{0594}', '\u{0595}', '\u{0597}',
    '\u{0598}', '\u{0599}', '\u{059C}', '\u{059D}', '\u{059E}', '\u{059F}', '\u{05A0}', '\u{05A1}',
    '\u{05A8}', '\u{05A9}', '\u{05AB}', '\u{05AC}', '\u{05AF}', '\u{05C4}', '\u{0610}', '\u{0611}',
    '\u{0612}', '\u{0613}', '\u{0614}', '\u{0615}', '\u{0616}', '\u{0617}', '\u{0657}', '\u{0658}',
    '\u{0659}', '\u{065A}', '\u{065B}', '\u{065D}', '\u{065E}', '\u{06D6}', '\u{06D7}', '\u{06D8}',
    '\u{06D9}', '\u{06DA}', '\u{06DB}', '\u{06DC}', '\u{06DF}', '\u{06E0}', '\u{06E1}', '\u{06E2}',
    '\u{06E4}', '\u{06E7}', '\u{06E8}', '\u{06EB}', '\u{06EC}', '\u{0730}', '\u{0732}', '\u{0733}',
    '\u{0735}', '\u{0736}', '\u{073A}', '\u{073D}', '\u{073F}', '\u{0740}', '\u{0741}', '\u{0743}',
    '\u{0745}', '\u{0747}', '\u{0749}', '\u{074A}', '\u{07EB}', '\u{07EC}', '\u{07ED}', '\u{07EE}',
    '\u{07EF}', '\u{07F0}', '\u{07F1}', '\u{07F3}', '\u{0816}', '\u{0817}', '\u{0818}', '\u{0819}',
    '\u{081B}', '\u{081C}', '\u{081D}', '\u{081E}', '\u{081F}', '\u{0820}', '\u{0821}', '\u{0822}',
    '\u{0823}', '\u{0825}', '\u{0826}', '\u{0827}', '\u{0829}', '\u{082A}', '\u{082B}', '\u{082C}',
    '\u{082D}', '\u{0951}', '\u{0953}', '\u{0954}', '\u{0F82}', '\u{0F83}', '\u{0F86}', '\u{0F87}',
    '\u{135D}', '\u{135E}', '\u{135F}', '\u{17DD}', '\u{193A}', '\u{1A17}', '\u{1A75}', '\u{1A76}',
    '\u{1A77}', '\u{1A78}', '\u{1A79}', '\u{1A7A}', '\u{1A7B}', '\u{1A7C}', '\u{1B6B}', '\u{1B6D}',
    '\u{1B6E}', '\u{1B6F}', '\u{1B70}', '\u{1B71}', '\u{1B72}', '\u{1B73}', '\u{1CD0}', '\u{1CD1}',
    '\u{1CD2}', '\u{1CDA}', '\u{1CDB}', '\u{1CE0}', '\u{1DC0}', '\u{1DC1}', '\u{1DC3}', '\u{1DC4}',
    '\u{1DC5}', '\u{1DC6}', '\u{1DC7}', '\u{1DC8}', '\u{1DC9}', '\u{1DCB}', '\u{1DCC}', '\u{1DD1}',
    '\u{1DD2}', '\u{1DD3}', '\u{1DD4}', '\u{1DD5}', '\u{1DD6}', '\u{1DD7}', '\u{1DD8}', '\u{1DD9}',
    '\u{1DDA}', '\u{1DDB}', '\u{1DDC}', '\u{1DDD}', '\u{1DDE}', '\u{1DDF}', '\u{1DE0}', '\u{1DE1}',
    '\u{1DE2}', '\u{1DE3}', '\u{1DE4}', '\u{1DE5}', '\u{1DE6}', '\u{1DFE}', '\u{20D0}', '\u{20D1}',
    '\u{20D4}', '\u{20D5}', '\u{20D6}', '\u{20D7}', '\u{20DB}', '\u{20DC}', '\u{20E1}', '\u{20E7}',
    '\u{20E9}', '\u{20F0}', '\u{2CEF}', '\u{2CF0}', '\u{2CF1}', '\u{2DE0}', '\u{2DE1}', '\u{2DE2}',
    '\u{2DE3}', '\u{2DE4}', '\u{2DE5}', '\u{2DE6}', '\u{2DE7}', '\u{2DE8}', '\u{2DE9}', '\u{2DEA}',
    '\u{2DEB}', '\u{2DEC}', '\u{2DED}', '\u{2DEE}', '\u{2DEF}', '\u{2DF0}', '\u{2DF1}', '\u{2DF2}',
    '\u{2DF3}', '\u{2DF4}', '\u{2DF5}', '\u{2DF6}', '\u{2DF7}', '\u{2DF8}', '\u{2DF9}', '\u{2DFA}',
    '\u{2DFB}', '\u{2DFC}', '\u{2DFD}', '\u{2DFE}', '\u{2DFF}', '\u{A66F}', '\u{A67C}', '\u{A67D}',
    '\u{A6F0}', '\u{A6F1}', '\u{A8E0}', '\u{A8E1}', '\u{A8E2}', '\u{A8E3}', '\u{A8E4}', '\u{A8E5}',
    '\u{A8E6}', '\u{A8E7}', '\u{A8E8}', '\u{A8E9}', '\u{A8EA}', '\u{A8EB}', '\u{A8EC}', '\u{A8ED}',
    '\u{A8EE}', '\u{A8EF}', '\u{A8F0}', '\u{A8F1}', '\u{AAB0}', '\u{AAB2}', '\u{AAB3}', '\u{AAB7}',
    '\u{AAB8}', '\u{AABE}', '\u{AABF}', '\u{AAC1}', '\u{FE20}', '\u{FE21}', '\u{FE22}', '\u{FE23}',
    '\u{FE24}', '\u{FE25}', '\u{FE26}', '\u{10A0F}', '\u{10A38}', '\u{1D185}', '\u{1D186}', '\u{1D187}',
    '\u{1D188}', '\u{1D189}', '\u{1D1AA}', '\u{1D1AB}', '\u{1D1AC}', '\u{1D1AD}', '\u{1D242}', '\u{1D243}',
    '\u{1D244}',
];

/// First id handed out. Ids are 24-bit (the foreground colour), never 0.
const FIRST_ID: u32 = 0x00_7a_01;

/// One picture kitty holds for us.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Placed {
    id: u32,
    path: String,
    cols: u16,
    rows: u16,
}

pub struct Kitty {
    next_id: u32,
    placed: Vec<Placed>,
}

impl Kitty {
    pub fn new() -> Self {
        Self { next_id: FIRST_ID, placed: Vec::new() }
    }

    fn allocate(&mut self) -> u32 {
        let id = self.next_id;
        // Stay inside 24 bits and away from 0.
        self.next_id = if self.next_id >= 0x00ff_ffff { FIRST_ID } else { self.next_id + 1 };
        id
    }
}

impl Default for Kitty {
    fn default() -> Self {
        Self::new()
    }
}

impl Graphics for Kitty {
    fn place(&mut self, images: &[ImageRequest], buf: &mut Buffer, out: &mut dyn Write) -> io::Result<()> {
        let mut keep = Vec::with_capacity(images.len());
        for request in images {
            let cols = request.rect.width.min(DIACRITICS.len() as u16);
            let rows = request.rect.height.min(DIACRITICS.len() as u16);
            if cols == 0 || rows == 0 {
                continue;
            }
            // Reuse a placement for the same file at the same size; anything
            // else is a new upload, because a virtual placement's size is set
            // when it is created.
            let reused = self
                .placed
                .iter()
                .position(|p| p.path == request.path && p.cols == cols && p.rows == rows)
                .map(|i| self.placed.remove(i));
            let placed = match reused {
                Some(placed) => placed,
                None => {
                    let id = self.allocate();
                    out.write_all(transmit(id, &request.path, cols, rows).as_bytes())?;
                    Placed { id, path: request.path.clone(), cols, rows }
                }
            };
            paint(buf, Rect::new(request.rect.x, request.rect.y, cols, rows), placed.id);
            keep.push(placed);
        }
        for gone in self.placed.drain(..) {
            out.write_all(delete(gone.id).as_bytes())?;
        }
        self.placed = keep;
        out.flush()
    }

    fn clear(&mut self, out: &mut dyn Write) -> io::Result<()> {
        for gone in self.placed.drain(..) {
            out.write_all(delete(gone.id).as_bytes())?;
        }
        out.flush()
    }
}

/// Upload `path` as image `id` and create its virtual placement: quiet
/// (`q=2`, kitty answers nothing, so no reply lands in the key stream) and
/// without moving the cursor (`C=1`).
pub fn transmit(id: u32, path: &str, cols: u16, rows: u16) -> String {
    format!(
        "\x1b_Ga=T,t=f,f=100,U=1,q=2,C=1,i={id},c={cols},r={rows};{}\x1b\\",
        base64(path.as_bytes())
    )
}

/// Delete image `id`, its placements and its data.
pub fn delete(id: u32) -> String {
    format!("\x1b_Ga=d,d=I,i={id},q=2\x1b\\")
}

/// Fill `rect` with placeholder cells for image `id`: every cell carries its
/// own row and column, so ratatui's diff can resend any one of them alone.
pub fn paint(buf: &mut Buffer, rect: Rect, id: u32) {
    let fg = Color::Rgb((id >> 16) as u8, (id >> 8) as u8, id as u8);
    for row in 0..rect.height {
        for col in 0..rect.width {
            let Some(cell) = buf.cell_mut((rect.x + col, rect.y + row)) else {
                continue;
            };
            let mut symbol = String::with_capacity(12);
            symbol.push(PLACEHOLDER);
            symbol.push(DIACRITICS[row as usize]);
            symbol.push(DIACRITICS[col as usize]);
            cell.reset();
            cell.set_symbol(&symbol).set_fg(fg);
        }
    }
}

/// Standard base64 with padding — the path is the only payload, so a crate
/// for it would be all cost.
pub fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i)) as usize & 63] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}
