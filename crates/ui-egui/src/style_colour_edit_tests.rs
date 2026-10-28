//! #1140: the colour picker opened from the style editor's Properties row must survive an edit:
//! a drag in its saturation square redefines the style, and the editor and the picker both stay
//! open. On v0.10.0 the click inside the picker counted as a click outside the style editor, so
//! it closed the editor and the picker with it.

use egui::{Event, Pos2, Shape, pos2};
use figcraft_engine::Session;
use serde_json::json;

use crate::colorpicker::PickerTarget;
use crate::layers_drag_tests::Ui;
use crate::style_picker::{StyleSlot, open_for_selection};

/// The style's first paint as a hex string ("0D99FF").
fn hex(u: &Ui) -> String {
    let doc = u.app.session.doc().unwrap();
    let Some(figcraft_doc::StyleKind::Paint { paints }) = doc.style("Brand/Primary").map(|s| &s.kind) else { return String::new() };
    paints.first().and_then(|p| p.solid_color()).map(|c| c.to_hex()).unwrap_or_default()
}

/// The texts drawn in one frame, with their centres.
fn texts(u: &mut Ui) -> Vec<(String, Pos2)> {
    fn walk(shape: &Shape, out: &mut Vec<(String, Pos2)>) {
        match shape {
            Shape::Text(t) => out.push((t.galley.text().to_string(), t.pos + t.galley.size() / 2.0)),
            Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for c in u.frame_output(vec![]).shapes {
        walk(&c.shape, &mut out);
    }
    out
}

#[test]
fn style_colour_picker_survives_a_colour_edit() {
    let mut s = Session::new();
    s.execute("style.create", &json!({"type": "paint", "name": "Brand/Primary", "value": [{"type": "solid", "color": "#0D99FF"}]})).unwrap();
    let mut u = Ui::new(s);
    // The style editor, as the Local styles list opens it (nothing selected).
    open_for_selection(&mut u.app, StyleSlot::Fill, Some("Brand/Primary"));
    for _ in 0..3 {
        u.frame(vec![]);
    }
    // The colour row under Properties: "0D99FF  100%".
    let row = texts(&mut u).into_iter().find(|(t, _)| t.ends_with("  100%")).map(|(_, p)| p).expect("the colour row under Properties");
    u.frame(vec![Event::PointerMoved(row)]);
    u.button(row, true);
    u.button(row, false);
    u.frame(vec![]);
    assert!(matches!(u.app.ui.picker, Some(PickerTarget::Style { .. })), "clicking the colour opens the picker");
    // Drag inside the saturation square: top-left of the picker is where the picker puts itself.
    let at = pos2(1440.0 - u.app.ui.right_width - 260.0, 140.0);
    let (from, to) = (at + egui::vec2(60.0, 160.0), at + egui::vec2(150.0, 250.0));
    u.frame(vec![Event::PointerMoved(from)]);
    u.button(from, true);
    for k in 1..=5 {
        u.frame(vec![Event::PointerMoved(from + (to - from) * (k as f32 / 5.0))]);
    }
    u.button(to, false);
    u.frame(vec![]);
    assert_ne!(hex(&u), "0D99FF", "the drag redefines the style's colour (it is {})", hex(&u));
    assert!(u.app.ui.style_picker.is_some(), "the style editor stays open while its colour picker is used");
    assert!(matches!(u.app.ui.picker, Some(PickerTarget::Style { .. })), "the colour picker stays open too: {:?}", u.app.ui.picker);
}
