use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use lvgl;
use lvgl::widgets::Label;
use lvgl::{Display, DrawBuffer};
use parking_lot::Mutex;
use std::cell::RefCell;
use std::sync::Arc;

type ColorSpace = Rgb565;

fn main() {
    let embedded_graphics_display: SimulatorDisplay<ColorSpace> = SimulatorDisplay::new(Size::new(
        lvgl::DISP_HOR_RES as u32,
        lvgl::DISP_VER_RES as u32,
    ));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("App Example", &output_settings);

    let mut shared_native_display = Arc::new(Mutex::new(embedded_graphics_display));

    // LVGL usage
    lvgl::init();

    const REFRESH_BUFFER_SIZE: usize = lvgl::DISP_HOR_RES * lvgl::DISP_VER_RES / 10;
    static DRAW_BUFFER: DrawBuffer<REFRESH_BUFFER_SIZE> = DrawBuffer::new();

    let display = Display::register(&DRAW_BUFFER, {
        let shared_display = Arc::clone(&shared_native_display);
        move |update| {
            let mut embedded_graphics_display = shared_display.lock();
            embedded_graphics_display.draw_iter(update.as_pixels());
        }
    })
    .unwrap();

    let label: Label = "Nice!".into();
}
