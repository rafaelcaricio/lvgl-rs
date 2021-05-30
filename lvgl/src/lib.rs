//! [![github]](https://github.com/rafaelcaricio/lvgl-rs)&ensp;[![crates-io]](https://crates.io/crates/lvgl)&ensp;[![docs-rs]](crate)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! <br>
//!
//! [LVGL][1] bindings for Rust. A powerful and easy-to-use embedded GUI with many widgets, advanced visual effects, and
//! low memory footprint. This crate is compatible with `#![no_std]` environments by default.
//!
//! [1]: https://docs.lvgl.io/v7/en/html/get-started/quick-overview.html
//!

#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod lv_core;

#[cfg(feature = "lvgl_alloc")]
extern crate alloc;

// We can ONLY use `alloc::boxed::Box` if `lvgl_alloc` is enabled.
// That is because we use `Box` to send memory references to LVGL. Since the global allocator, when
// `lvgl_alloc` feature is enabled, is the LVGL memory manager then everything is in LVGL
// managed memory anyways. In that case we can use the Rust's provided Box definition.
//
#[cfg(feature = "lvgl_alloc")]
use ::alloc::boxed::Box;

#[cfg(feature = "lvgl_alloc")]
mod allocator;

#[cfg(not(feature = "lvgl_alloc"))]
pub(crate) mod mem;

// When LVGL allocator is not used on the Rust code, we need a way to add objects to the LVGL
// managed memory. We implement a very simple `Box` that has the minimal features to copy memory
// safely to the LVGL managed memory.
//
#[cfg(not(feature = "lvgl_alloc"))]
use crate::mem::Box;

pub mod display;
mod functions;
mod support;
mod ui;
pub mod widgets;
use core::sync::atomic::{AtomicBool, Ordering};
pub use functions::*;
pub use lv_core::*;
use parking_lot::Mutex;
pub use support::*;
pub use ui::*;

lazy_static! {
    static ref MUTEX: Mutex<AtomicBool> = Mutex::new(AtomicBool::new(false));
}

pub fn init() {
    let initialized = MUTEX.lock();
    if initialized
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        unsafe {
            lvgl_sys::lv_init();
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::display::Display;
    use embedded_graphics::mock_display::MockDisplay;
    use embedded_graphics::pixelcolor::Rgb565;

    pub(crate) fn initialize_test() {
        init();

        static RUN_ONCE: Mutex<Option<u8>> = parking_lot::const_mutex(None);
        let mut run_once = RUN_ONCE.lock();

        if run_once.is_none() {
            let embedded_graphics_display: MockDisplay<Rgb565> = Default::default();
            let _ = Display::register(embedded_graphics_display).unwrap();
            *run_once = Some(1);
        }
    }
}
