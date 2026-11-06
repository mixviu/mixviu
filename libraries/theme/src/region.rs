// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Region capture colours resolved from design roles, independent of egui.
use crate::Theme;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colours {
    pub outline: [u8; 4],
    pub back: [u8; 4],
    pub edge: [u8; 4],
    pub text: [u8; 4],
}

pub const DIM_ALPHA: u8 = 115;
pub const BORDER_WIDTH: f32 = 2.0;

pub fn colours(theme: &Theme) -> Colours {
    let dictionary = theme.dictionary();
    let dark = theme.mode() == design::Mode::Dark;
    let grey = |value| [value, value, value, 255];
    let pair = |name: &str, foreground: bool| {
        dictionary.colours.pairs.get(name).map(|pair| {
            if foreground { pair.rendered_foreground } else { pair.rendered_surface }.to_srgba8()
        })
    };
    let chrome = |name: &str| {
        theme.overlay().chrome.get(name).map(|v| v.0).or_else(|| {
            dictionary.chrome.as_ref().and_then(|roles| roles.get(name)).map(|v| v.to_srgba8())
        })
    };
    let chrome_widgets = theme.style().widgets == design::family::style::Widgets::Chrome;
    let mut result = Colours {
        outline: dictionary.colours.non_text.get("ring").map(|v| v.value.to_srgba8())
            .or_else(|| theme.accent().map(|v| v.to_srgba8()))
            .unwrap_or(if dark { [0, 92, 128, 255] } else { [144, 209, 255, 255] }),
        back: pair("popover", false).unwrap_or(grey(if dark { 27 } else { 248 })),
        edge: dictionary.colours.non_text.get("border").map(|v| v.value.to_srgba8())
            .unwrap_or(grey(if dark { 60 } else { 190 })),
        text: pair("secondary", true).unwrap_or(grey(if dark { 255 } else { 0 })),
    };
    if chrome_widgets {
        if let Some(value) = chrome("card") { result.back = value; }
        if let Some(value) = chrome("card_border") { result.edge = value; }
        if let Some(value) = chrome("text") { result.text = value; }
    } else {
        for (role, destination) in [("card", &mut result.back), ("card_border", &mut result.edge), ("text", &mut result.text)] {
            if let Some(value) = theme.overlay().chrome.get(role) { *destination = value.0; }
        }
    }
    result
}
