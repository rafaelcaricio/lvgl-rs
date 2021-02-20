#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate bitflags;

pub mod input_device;
pub(crate) mod mem;
mod support;
mod ui;
#[macro_use]
mod lv_core;
pub mod widgets;

pub use lv_core::*;
pub use support::*;
pub use ui::*;

pub const HOR_RES_MAX: u32 = lvgl_sys::LV_HOR_RES_MAX;
pub const VER_RES_MAX: u32 = lvgl_sys::LV_VER_RES_MAX;
