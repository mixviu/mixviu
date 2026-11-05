// SPDX-License-Identifier: MIT OR Apache-2.0
//! Pinned shared Lucide assets, independent of any renderer or application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Icon {
    name: &'static str,
    svg: &'static str,
}
impl Icon {
    pub fn name(self) -> &'static str {
        self.name
    }
    pub fn svg(self) -> &'static str {
        self.svg
    }
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|i| i.name == name)
    }
    pub const ALL: &[Self] = &[
        Icon {
            name: "accessibility",
            svg: include_str!("../assets/accessibility.svg"),
        },
        Icon {
            name: "app-window",
            svg: include_str!("../assets/app-window.svg"),
        },
        Icon {
            name: "arrow-down",
            svg: include_str!("../assets/arrow-down.svg"),
        },
        Icon {
            name: "arrow-left",
            svg: include_str!("../assets/arrow-left.svg"),
        },
        Icon {
            name: "arrow-right",
            svg: include_str!("../assets/arrow-right.svg"),
        },
        Icon {
            name: "arrow-up",
            svg: include_str!("../assets/arrow-up.svg"),
        },
        Icon {
            name: "book-open",
            svg: include_str!("../assets/book-open.svg"),
        },
        Icon {
            name: "book-user",
            svg: include_str!("../assets/book-user.svg"),
        },
        Icon {
            name: "calendar",
            svg: include_str!("../assets/calendar.svg"),
        },
        Icon {
            name: "camera",
            svg: include_str!("../assets/camera.svg"),
        },
        Icon {
            name: "check",
            svg: include_str!("../assets/check.svg"),
        },
        Icon {
            name: "chevron-down",
            svg: include_str!("../assets/chevron-down.svg"),
        },
        Icon {
            name: "chevron-left",
            svg: include_str!("../assets/chevron-left.svg"),
        },
        Icon {
            name: "chevron-right",
            svg: include_str!("../assets/chevron-right.svg"),
        },
        Icon {
            name: "chevrons-left",
            svg: include_str!("../assets/chevrons-left.svg"),
        },
        Icon {
            name: "chevrons-right",
            svg: include_str!("../assets/chevrons-right.svg"),
        },
        Icon {
            name: "circle-alert",
            svg: include_str!("../assets/circle-alert.svg"),
        },
        Icon {
            name: "circle-check",
            svg: include_str!("../assets/circle-check.svg"),
        },
        Icon {
            name: "circle",
            svg: include_str!("../assets/circle.svg"),
        },
        Icon {
            name: "contact",
            svg: include_str!("../assets/contact.svg"),
        },
        Icon {
            name: "copy",
            svg: include_str!("../assets/copy.svg"),
        },
        Icon {
            name: "cpu",
            svg: include_str!("../assets/cpu.svg"),
        },
        Icon {
            name: "crop",
            svg: include_str!("../assets/crop.svg"),
        },
        Icon {
            name: "disc",
            svg: include_str!("../assets/disc.svg"),
        },
        Icon {
            name: "download",
            svg: include_str!("../assets/download.svg"),
        },
        Icon {
            name: "ellipsis",
            svg: include_str!("../assets/ellipsis.svg"),
        },
        Icon {
            name: "external-link",
            svg: include_str!("../assets/external-link.svg"),
        },
        Icon {
            name: "eye-off",
            svg: include_str!("../assets/eye-off.svg"),
        },
        Icon {
            name: "eye",
            svg: include_str!("../assets/eye.svg"),
        },
        Icon {
            name: "file-archive",
            svg: include_str!("../assets/file-archive.svg"),
        },
        Icon {
            name: "file-code",
            svg: include_str!("../assets/file-code.svg"),
        },
        Icon {
            name: "file-cog",
            svg: include_str!("../assets/file-cog.svg"),
        },
        Icon {
            name: "file-image",
            svg: include_str!("../assets/file-image.svg"),
        },
        Icon {
            name: "file-music",
            svg: include_str!("../assets/file-music.svg"),
        },
        Icon {
            name: "file-spreadsheet",
            svg: include_str!("../assets/file-spreadsheet.svg"),
        },
        Icon {
            name: "file-terminal",
            svg: include_str!("../assets/file-terminal.svg"),
        },
        Icon {
            name: "file-text",
            svg: include_str!("../assets/file-text.svg"),
        },
        Icon {
            name: "file-type",
            svg: include_str!("../assets/file-type.svg"),
        },
        Icon {
            name: "file-video-camera",
            svg: include_str!("../assets/file-video-camera.svg"),
        },
        Icon {
            name: "file",
            svg: include_str!("../assets/file.svg"),
        },
        Icon {
            name: "film",
            svg: include_str!("../assets/film.svg"),
        },
        Icon {
            name: "flag",
            svg: include_str!("../assets/flag.svg"),
        },
        Icon {
            name: "folder-input",
            svg: include_str!("../assets/folder-input.svg"),
        },
        Icon {
            name: "folder-open",
            svg: include_str!("../assets/folder-open.svg"),
        },
        Icon {
            name: "folder-plus",
            svg: include_str!("../assets/folder-plus.svg"),
        },
        Icon {
            name: "folder",
            svg: include_str!("../assets/folder.svg"),
        },
        Icon {
            name: "forward",
            svg: include_str!("../assets/forward.svg"),
        },
        Icon {
            name: "globe",
            svg: include_str!("../assets/globe.svg"),
        },
        Icon {
            name: "hard-drive",
            svg: include_str!("../assets/hard-drive.svg"),
        },
        Icon {
            name: "hash",
            svg: include_str!("../assets/hash.svg"),
        },
        Icon {
            name: "highlighter",
            svg: include_str!("../assets/highlighter.svg"),
        },
        Icon {
            name: "house",
            svg: include_str!("../assets/house.svg"),
        },
        Icon {
            name: "image",
            svg: include_str!("../assets/image.svg"),
        },
        Icon {
            name: "inbox",
            svg: include_str!("../assets/inbox.svg"),
        },
        Icon {
            name: "info",
            svg: include_str!("../assets/info.svg"),
        },
        Icon {
            name: "keyboard",
            svg: include_str!("../assets/keyboard.svg"),
        },
        Icon {
            name: "layers",
            svg: include_str!("../assets/layers.svg"),
        },
        Icon {
            name: "layout-template",
            svg: include_str!("../assets/layout-template.svg"),
        },
        Icon {
            name: "loader-circle",
            svg: include_str!("../assets/loader-circle.svg"),
        },
        Icon {
            name: "mail-open",
            svg: include_str!("../assets/mail-open.svg"),
        },
        Icon {
            name: "mail",
            svg: include_str!("../assets/mail.svg"),
        },
        Icon {
            name: "menu",
            svg: include_str!("../assets/menu.svg"),
        },
        Icon {
            name: "message-square",
            svg: include_str!("../assets/message-square.svg"),
        },
        Icon {
            name: "minus",
            svg: include_str!("../assets/minus.svg"),
        },
        Icon {
            name: "monitor",
            svg: include_str!("../assets/monitor.svg"),
        },
        Icon {
            name: "moon",
            svg: include_str!("../assets/moon.svg"),
        },
        Icon {
            name: "mouse-pointer-2",
            svg: include_str!("../assets/mouse-pointer-2.svg"),
        },
        Icon {
            name: "move-up-right",
            svg: include_str!("../assets/move-up-right.svg"),
        },
        Icon {
            name: "music",
            svg: include_str!("../assets/music.svg"),
        },
        Icon {
            name: "network",
            svg: include_str!("../assets/network.svg"),
        },
        Icon {
            name: "package",
            svg: include_str!("../assets/package.svg"),
        },
        Icon {
            name: "palette",
            svg: include_str!("../assets/palette.svg"),
        },
        Icon {
            name: "panel-bottom",
            svg: include_str!("../assets/panel-bottom.svg"),
        },
        Icon {
            name: "panel-left",
            svg: include_str!("../assets/panel-left.svg"),
        },
        Icon {
            name: "panel-right",
            svg: include_str!("../assets/panel-right.svg"),
        },
        Icon {
            name: "paperclip",
            svg: include_str!("../assets/paperclip.svg"),
        },
        Icon {
            name: "pencil",
            svg: include_str!("../assets/pencil.svg"),
        },
        Icon {
            name: "pin-off",
            svg: include_str!("../assets/pin-off.svg"),
        },
        Icon {
            name: "pin",
            svg: include_str!("../assets/pin.svg"),
        },
        Icon {
            name: "play",
            svg: include_str!("../assets/play.svg"),
        },
        Icon {
            name: "plug",
            svg: include_str!("../assets/plug.svg"),
        },
        Icon {
            name: "presentation",
            svg: include_str!("../assets/presentation.svg"),
        },
        Icon {
            name: "printer",
            svg: include_str!("../assets/printer.svg"),
        },
        Icon {
            name: "redo-2",
            svg: include_str!("../assets/redo-2.svg"),
        },
        Icon {
            name: "refresh-cw",
            svg: include_str!("../assets/refresh-cw.svg"),
        },
        Icon {
            name: "reply-all",
            svg: include_str!("../assets/reply-all.svg"),
        },
        Icon {
            name: "reply",
            svg: include_str!("../assets/reply.svg"),
        },
        Icon {
            name: "save",
            svg: include_str!("../assets/save.svg"),
        },
        Icon {
            name: "scan",
            svg: include_str!("../assets/scan.svg"),
        },
        Icon {
            name: "scroll-text",
            svg: include_str!("../assets/scroll-text.svg"),
        },
        Icon {
            name: "search",
            svg: include_str!("../assets/search.svg"),
        },
        Icon {
            name: "send",
            svg: include_str!("../assets/send.svg"),
        },
        Icon {
            name: "server",
            svg: include_str!("../assets/server.svg"),
        },
        Icon {
            name: "settings",
            svg: include_str!("../assets/settings.svg"),
        },
        Icon {
            name: "share-2",
            svg: include_str!("../assets/share-2.svg"),
        },
        Icon {
            name: "shield-alert",
            svg: include_str!("../assets/shield-alert.svg"),
        },
        Icon {
            name: "square-dashed",
            svg: include_str!("../assets/square-dashed.svg"),
        },
        Icon {
            name: "square-pen",
            svg: include_str!("../assets/square-pen.svg"),
        },
        Icon {
            name: "square",
            svg: include_str!("../assets/square.svg"),
        },
        Icon {
            name: "star",
            svg: include_str!("../assets/star.svg"),
        },
        Icon {
            name: "sun",
            svg: include_str!("../assets/sun.svg"),
        },
        Icon {
            name: "trash",
            svg: include_str!("../assets/trash.svg"),
        },
        Icon {
            name: "type",
            svg: include_str!("../assets/type.svg"),
        },
        Icon {
            name: "undo-2",
            svg: include_str!("../assets/undo-2.svg"),
        },
        Icon {
            name: "unplug",
            svg: include_str!("../assets/unplug.svg"),
        },
        Icon {
            name: "x",
            svg: include_str!("../assets/x.svg"),
        },
        Icon {
            name: "zap",
            svg: include_str!("../assets/zap.svg"),
        },
        Icon {
            name: "zoom-in",
            svg: include_str!("../assets/zoom-in.svg"),
        },
        Icon {
            name: "zoom-out",
            svg: include_str!("../assets/zoom-out.svg"),
        },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_round_trip_and_sources_are_vectors() {
        for (n, i) in Icon::ALL.iter().enumerate() {
            assert_eq!(Icon::from_name(i.name()), Some(*i));
            assert!(!Icon::ALL[..n].iter().any(|p| p.name() == i.name()));
            assert!(i.svg().contains("<svg"));
        }
        assert!(Icon::from_name("missing").is_none());
    }
}
