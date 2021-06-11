use cstr_core::CString;
use embedded_graphics::drawable;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl;
use lvgl::style::Style;
use lvgl::widgets::{Label, LabelAlign};
use lvgl::{Align, Color, DefaultDisplay, Display, DrawBuffer, LvError, Part, State, Widget};
use lvgl_sys;
use parking_lot::Mutex;
use std::sync::Arc as SyncArc;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() -> Result<(), LvError> {
    let embedded_graphics_display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(
        lvgl::DISP_HOR_RES as u32,
        lvgl::DISP_VER_RES as u32,
    ));

    let output_settings = OutputSettingsBuilder::new().scale(1).build();
    let mut window = Window::new("PineTime", &output_settings);

    let shared_native_display = SyncArc::new(Mutex::new(embedded_graphics_display));

    // LVGL-rs usage starts here
    lvgl::init();

    // LVGL will render the graphics here first, and seed the rendered image to the
    // display. The buffer size can be set freely but 1/10 screen size is a good starting point.
    const REFRESH_BUFFER_SIZE: usize = lvgl::DISP_HOR_RES * lvgl::DISP_VER_RES / 10;
    static DRAW_BUFFER: DrawBuffer<REFRESH_BUFFER_SIZE> = DrawBuffer::new();
    //
    // const NUMBER_OF_DISPLAYS: usize = 1;
    // static DISPLAY_REGISTRY: DisplayRegistry<NUMBER_OF_DISPLAYS> = DisplayRegistry::empty();
    // // static DISPLAY_REGISTRY: SingleDisplayRegistry = DisplayRegistry::empty();
    // let display = DISPLAY_REGISTRY.register_shared(&DRAW_BUFFER, shared_native_display.clone())?;

    // Register your display update callback with LVGL. The closure you pass here will be called
    // whenever LVGL has updates to be painted to the display.
    let display = Display::register(&DRAW_BUFFER, {
        let shared_disp_inner = SyncArc::clone(&shared_native_display);
        move |update| {
            let mut em_disp = shared_disp_inner.lock();
            em_disp.draw_iter(update.as_pixels());
        }
    })?;

    // Create screen and widgets
    let mut screen = display.get_scr_act()?;

    println!("Before all widgets: {:?}", mem_info());

    let mut screen_style = Style::default();
    screen_style.set_bg_color(State::DEFAULT, Color::from_rgb((0, 0, 0)));
    screen_style.set_radius(State::DEFAULT, 0);
    screen.add_style(Part::Main, &mut screen_style)?;

    let mut time = Label::from("20:46");
    let mut style_time = Style::default();
    // style_time.set_text_font(font_noto_sans_numeric_28);
    style_time.set_text_color(State::DEFAULT, Color::from_rgb((255, 255, 255)));
    time.add_style(Part::Main, &mut style_time)?;
    time.set_align(&mut screen, Align::Center, 0, 0)?;
    time.set_width(240)?;
    time.set_height(240)?;

    let mut bt = Label::from("#5794f2 \u{F293}#");
    bt.set_width(50)?;
    bt.set_height(80)?;
    bt.set_recolor(true)?;
    bt.set_label_align(LabelAlign::Left)?;
    bt.set_align(&mut screen, Align::InTopLeft, 0, 0)?;

    let mut power: Label = "#fade2a 20%#".into();
    power.set_recolor(true)?;
    power.set_width(80)?;
    power.set_height(20)?;
    power.set_label_align(LabelAlign::Right)?;
    power.set_align(&mut screen, Align::InTopRight, 0, 0)?;

    // LVGL timer thread
    thread::spawn(|| {
        let interval = Duration::from_millis(5);
        loop {
            thread::sleep(interval);
            lvgl::tick_inc(interval);
        }
    });

    let mut i = 0;
    'running: loop {
        if i > 59 {
            i = 0;
        }
        let val = CString::new(format!("21:{:02}", i)).unwrap();
        time.set_text(&val)?;
        i = 1 + i;

        lvgl::task_handler();
        {
            let native_display = shared_native_display.lock();
            window.update(&native_display);
        }

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }
        println!("During run: {:?}", mem_info());
        sleep(Duration::from_secs(1));
    }

    println!("Final part of demo app: {:?}", mem_info());

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

fn mem_info() -> lvgl_sys::lv_mem_monitor_t {
    let mut info = lvgl_sys::lv_mem_monitor_t {
        total_size: 0,
        free_cnt: 0,
        free_size: 0,
        free_biggest_size: 0,
        used_cnt: 0,
        max_used: 0,
        used_pct: 0,
        frag_pct: 0,
    };
    unsafe {
        lvgl_sys::lv_mem_monitor(&mut info as *mut _);
    }
    info
}
