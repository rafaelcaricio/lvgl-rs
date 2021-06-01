use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use lvgl;
use lvgl::display::Display;
use lvgl::widgets::Label;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::sync::Arc;

type ColorSpace = Rgb565;

fn main() {
    let embedded_graphics_display: SimulatorDisplay<ColorSpace> = SimulatorDisplay::new(Size::new(
        lvgl_sys::LV_HOR_RES_MAX,
        lvgl_sys::LV_VER_RES_MAX,
    ));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("App Example", &output_settings);

    let mut shared_native_display = Arc::new(Mutex::new(embedded_graphics_display));

    // LVGL usage
    lvgl::init();
    let display = Display::register_shared(&shared_native_display).unwrap();
    let label = Label::new().unwrap();

    {
        let mut val = shared_native_display.lock();
        val.draw_pixel(Pixel::default()).unwrap();
    }
}
