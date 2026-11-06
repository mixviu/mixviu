// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lab client scene runtime. The evaluator is created and retained on its worker.
pub mod app_facilities;
pub mod app_host;
pub mod appearance;
pub mod bundle;
pub mod capture;
pub mod chooser;
pub mod debug;
mod debug_recent;
mod design_style;
pub mod documents;
pub mod editor;
pub mod elide;
pub mod file_data;
pub mod fonts;
pub mod gold_frame;
pub mod hosting;
pub mod http;
pub mod icons;
pub mod image_view;
pub mod jobs;
mod launch;
mod local_ui;
pub mod macro_host;
pub mod native_bus;
mod native_events;
pub mod native_menu;
mod native_primitives;
mod native_theme;
pub mod raster;
pub mod raster_decode;
pub mod rich;
pub mod runtime;
pub mod session;
mod shortcuts;
pub mod templates;
pub mod thumbnail;
pub mod timers;
pub mod tree_table;
pub mod view;
pub mod widgets;
pub mod window_frame;

pub mod calls;
pub mod decorations;
pub mod input;
pub mod layer;
pub mod platform;
pub mod ui_state;

pub mod editor_controls;
pub mod header_controls;

pub mod process;

pub mod time_series;

pub mod dashboard;
