// SPDX-License-Identifier: MIT OR Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelCommand {
    Show,
    Hide,
    Toggle,
    Pin,
    Unpin,
    Dock,
}
