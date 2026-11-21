//! The kitty backend's bytes and cells, without a terminal: what it writes to
//! the tty and what it leaves in the frame's buffer (docs/specs/25-image-pane.md).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use tuipalette_core::ImageRequest;
use tuipalette_tui::graphics::kitty::{self, Kitty, DIACRITICS};
use tuipalette_tui::graphics::{Graphics, Protocol};

fn request(path: &str, rect: Rect) -> ImageRequest {
    ImageRequest { rect, path: path.to_owned() }
}

fn ids_in(text: &str, key: &str) -> Vec<u32> {
    text.split("\x1b_G")
        .filter(|chunk| chunk.starts_with(key))
        .filter_map(|chunk| chunk.split(",i=").nth(1)?.split([',', ';']).next()?.parse().ok())
        .collect()
}

#[test]
fn base64_matches_the_rfc_vectors() {
    for (input, expected) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foobar", "Zm9vYmFy")] {
        assert_eq!(kitty::base64(input.as_bytes()), expected, "{input:?}");
    }
}

#[test]
fn the_diacritic_table_is_kittys() {
    assert_eq!(DIACRITICS.len(), 297);
    assert_eq!(&DIACRITICS[..3], &['\u{0305}', '\u{030D}', '\u{030E}']);
}

#[test]
fn a_transmit_names_the_file_and_the_cell_box_and_asks_for_no_reply() {
    let text = kitty::transmit(42, "/a.png", 6, 3);
    assert_eq!(text, "\x1b_Ga=T,t=f,f=100,U=1,q=2,C=1,i=42,c=6,r=3;L2EucG5n\x1b\\");
}

/// Every cell of the rect becomes a placeholder carrying its own row and
/// column, coloured with the image id; nothing outside the rect is touched.
#[test]
fn placing_fills_the_rect_with_placeholders_in_the_id_colour() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 6, 4));
    let mut out = Vec::new();
    let mut backend = Kitty::new();
    backend.place(&[request("/a.png", Rect::new(1, 1, 3, 2))], &mut buf, &mut out).expect("place");
    let id = ids_in(&String::from_utf8(out).expect("utf8"), "a=T")[0];
    let cell = &buf[(2, 2)];
    assert_eq!(cell.symbol(), "\u{10EEEE}\u{030D}\u{030D}", "row 1, column 1");
    assert_eq!(cell.fg, Color::Rgb((id >> 16) as u8, (id >> 8) as u8, id as u8));
    assert_eq!(buf[(0, 0)].symbol(), " ");
    assert_eq!(buf[(4, 1)].symbol(), " ");
}

/// The same picture in the same box is uploaded once; a different picture
/// replaces it and the old one is deleted; an empty list deletes everything.
#[test]
fn a_picture_is_uploaded_once_and_deleted_when_it_is_no_longer_asked_for() {
    let mut backend = Kitty::new();
    let area = Rect::new(0, 0, 8, 4);
    let mut frame = |images: &[ImageRequest]| {
        let mut buf = Buffer::empty(area);
        let mut out = Vec::new();
        backend.place(images, &mut buf, &mut out).expect("place");
        String::from_utf8(out).expect("utf8")
    };
    let a = request("/a.png", area);
    let first = frame(std::slice::from_ref(&a));
    assert_eq!(ids_in(&first, "a=T").len(), 1);
    let again = frame(std::slice::from_ref(&a));
    assert!(again.is_empty(), "the same picture again sends nothing: {again:?}");
    let swapped = frame(&[request("/b.png", area)]);
    assert_eq!(ids_in(&swapped, "a=d"), ids_in(&first, "a=T"), "a's id is deleted");
    assert_eq!(ids_in(&swapped, "a=T").len(), 1, "b is uploaded");
    let gone = frame(&[]);
    assert_eq!(ids_in(&gone, "a=d"), ids_in(&swapped, "a=T"));
}

/// The pane grew or shrank: a virtual placement's size is fixed, so the
/// picture is placed again at the new size and the old one deleted.
#[test]
fn a_new_size_is_a_new_placement() {
    let mut backend = Kitty::new();
    let mut out = Vec::new();
    let mut buf = Buffer::empty(Rect::new(0, 0, 10, 10));
    backend.place(&[request("/a.png", Rect::new(0, 0, 4, 4))], &mut buf, &mut out).expect("place");
    let first = ids_in(&String::from_utf8(std::mem::take(&mut out)).expect("utf8"), "a=T");
    backend.place(&[request("/a.png", Rect::new(0, 0, 6, 5))], &mut buf, &mut out).expect("place");
    let text = String::from_utf8(out).expect("utf8");
    assert_eq!(ids_in(&text, "a=d"), first);
    assert!(text.contains(",c=6,r=5;"), "{text:?}");
}

#[test]
fn clear_deletes_what_is_up_and_is_safe_twice() {
    let mut backend = Kitty::new();
    let mut buf = Buffer::empty(Rect::new(0, 0, 4, 4));
    let mut out = Vec::new();
    backend.place(&[request("/a.png", buf.area)], &mut buf, &mut out).expect("place");
    let placed = ids_in(&String::from_utf8(out).expect("utf8"), "a=T");
    let mut out = Vec::new();
    backend.clear(&mut out).expect("clear");
    assert_eq!(ids_in(&String::from_utf8(out).expect("utf8"), "a=d"), placed);
    let mut out = Vec::new();
    backend.clear(&mut out).expect("clear");
    assert!(out.is_empty());
}

#[test]
fn none_turns_pictures_off_and_kitty_is_always_kitty() {
    assert!(tuipalette_tui::graphics::detect(Protocol::None).is_none());
    assert!(tuipalette_tui::graphics::detect(Protocol::Kitty).is_some());
}
