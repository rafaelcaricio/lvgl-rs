use cstr_core::{CStr, CString};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl;
use lvgl::style::Style;
use lvgl::widgets::{Label, LabelAlign};
use lvgl::{Align, Color, LvError, Part, State, Widget, UI};
use lvgl_sys;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() -> Result<(), LvError> {
    let display: SimulatorDisplay<Rgb565> =
        SimulatorDisplay::new(Size::new(lvgl::HOR_RES_MAX, lvgl::VER_RES_MAX));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("PineTime", &output_settings);

    let mut ui = UI::init()?;

    // Implement and register your display:
    ui.disp_drv_register(display).unwrap();

    // Create screen and widgets
    let mut screen = ui.scr_act()?;

    let mut screen_style = Style::default();
    screen_style.set_bg_color(State::DEFAULT, Color::from_rgb((0, 0, 0)));
    screen_style.set_radius(State::DEFAULT, 0);
    screen.add_style(Part::Main, screen_style)?;

    let mut time = Label::new(&mut screen)?;
    let mut style_time = Style::default();
    //style_time.set_text_font(font_noto_sans_numeric_28);
    style_time.set_text_color(State::DEFAULT, Color::from_rgb((255, 255, 255)));
    time.add_style(Part::Main, style_time)?;
    time.set_align(&mut screen, Align::Center, 0, 0)?;
    time.set_text(CString::new("20:46").unwrap().as_c_str())?;
    time.set_width(240)?;
    time.set_height(240)?;

    let mut bt = Label::new(&mut screen)?;
    bt.set_width(50)?;
    bt.set_height(80)?;
    bt.set_recolor(true)?;
    bt.set_text(CString::new("#5794f2 \u{F293}#").unwrap().as_c_str())?;
    bt.set_label_align(LabelAlign::Left)?;
    bt.set_align(&mut screen, Align::InTopLeft, 0, 0)?;

    fn set_text<S>(text: S) -> Result<(), ()>
    where
        S: AsRef<cstr_core::CStr>,
    {
        let _v: *const i8 = text.as_ref().as_ptr();
        Ok(())
    }

    let mut t: heapless::String<heapless::consts::U8> = heapless::String::from("test");
    t.push('\0').unwrap();
    set_text(CStr::from_bytes_with_nul(t.as_bytes()).unwrap()).unwrap();
    set_text(cstr_core::CStr::from_bytes_with_nul("test\0".as_bytes()).unwrap()).unwrap();
    set_text(cstr_core::CString::new("test").unwrap().as_c_str()).unwrap();

    let mut power = Label::new(&mut screen)?;
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
        let val = format!("21:{:02}", i);
        time.set_text(CString::new(val.as_str()).unwrap().as_c_str())?;
        i = 1 + i;

        ui.task_handler();
        window.update(ui.get_display_ref().unwrap());

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }
        sleep(Duration::from_secs(1));

        ui.tick_inc(loop_started.elapsed());
        loop_started = Instant::now();
    }

    Ok(())
}

// Reference to native font for LittlevGL, defined in the file: "fonts_noto_sans_numeric_80.c"
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
