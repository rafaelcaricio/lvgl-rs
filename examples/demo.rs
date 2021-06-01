use cstr_core::CString;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl;
use lvgl::display::{DefaultDisplay, Display};
use lvgl::style::Style;
use lvgl::widgets::{Label, LabelAlign};
use lvgl::{Align, Color, LvError, Part, State, Widget, UI};
use lvgl_sys;
use parking_lot::Mutex;
use std::sync::Arc as SyncArc;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() -> Result<(), LvError> {
    let embedded_graphics_display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(
        lvgl_sys::LV_HOR_RES_MAX,
        lvgl_sys::LV_VER_RES_MAX,
    ));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("PineTime", &output_settings);

    lvgl::init();
    // let mut ui = UI::init()?;

    // Implement and register your display:
    let shared_native_display = SyncArc::new(Mutex::new(embedded_graphics_display));
    let _display = Display::register_shared(&shared_native_display)?;
    // ui.disp_drv_register(embedded_graphics_display).unwrap();

    // Create screen and widgets
    let mut screen = DefaultDisplay::get_scr_act()?;

    let mut screen_style = Style::default();
    screen_style.set_bg_color(State::DEFAULT, Color::from_rgb((0, 0, 0)));
    screen_style.set_radius(State::DEFAULT, 0);
    screen.add_style(Part::Main, &mut screen_style)?;

    let mut time = Label::new()?;
    let mut style_time = Style::default();
    // style_time.set_text_font(font_noto_sans_numeric_28);
    style_time.set_text_color(State::DEFAULT, Color::from_rgb((255, 255, 255)));
    time.add_style(Part::Main, &mut style_time)?;
    time.set_align(&mut screen, Align::Center, 0, 0)?;
    time.set_text(CString::new("20:46").unwrap().as_c_str())?;
    time.set_width(240)?;
    time.set_height(240)?;

    let mut bt = Label::new()?;
    bt.set_width(50)?;
    bt.set_height(80)?;
    bt.set_recolor(true)?;
    bt.set_text(CString::new("#5794f2 \u{F293}#").unwrap().as_c_str())?;
    bt.set_label_align(LabelAlign::Left)?;
    bt.set_align(&mut screen, Align::InTopLeft, 0, 0)?;

    let mut power = Label::new()?;
    power.set_recolor(true)?;
    power.set_width(80)?;
    power.set_height(20)?;
    power.set_text(CString::new("#fade2a 20%#").unwrap().as_c_str())?;
    power.set_label_align(LabelAlign::Right)?;
    power.set_align(&mut screen, Align::InTopRight, 0, 0)?;

    let mut i = 0;
    let mut loop_started = Instant::now();
    'running: loop {
        if i > 59 {
            i = 0;
        }
        println!("i = {}", i);
        let val = CString::new(format!("21:{:02}", i)).unwrap();
        time.set_text(&val)?;
        i = 1 + i;

        let wait_for = lvgl::task_handler();
        println!("wait for = {} ms", wait_for.unwrap().as_secs());
        {
            let native_display = shared_native_display.lock();
            window.update(&native_display);
        }
        // ui.task_handler();
        // window.update(ui.get_display_ref().unwrap());
        lvgl::tick_inc(loop_started.elapsed());

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }
        sleep(Duration::from_secs(1));

        loop_started = Instant::now();
    }

    Ok(())
}

// Reference to native font for LVGL, defined in the file: "fonts_noto_sans_numeric_80.c"
// TODO: Create a macro for defining a safe wrapper for fonts.
// Maybe sometihng like:
//
// font_declare! {
//     NotoSansNumeric80 = noto_sans_numeric_80;
// };
//
extern "C" {
    pub static mut noto_sans_numeric_80: lvgl_sys::lv_font_t;
}
