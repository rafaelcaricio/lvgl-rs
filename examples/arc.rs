use cstr_core::CString;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl::display::Display;
use lvgl::style::Style;
use lvgl::widgets::{Arc, Label, LabelAlign};
use lvgl::{self, Align, Color, Part, State};
use lvgl::{LvError, Widget};
use lvgl_sys;
use parking_lot::Mutex;
use std::sync::Arc as SyncArc;
use std::thread;
use std::time::{Duration, Instant};

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

fn main() -> Result<(), LvError> {
    println!("meminfo init: {:?}", mem_info());
    run_arc_demo()?;
    println!("meminfo end: {:?}", mem_info());
    Ok(())
}

fn run_arc_demo() -> Result<(), LvError> {
    lvgl::init();

    let embedded_graphics_display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(
        lvgl_sys::LV_HOR_RES_MAX,
        lvgl_sys::LV_VER_RES_MAX,
    ));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Arc Example", &output_settings);

    let shared_native_display = SyncArc::new(Mutex::new(embedded_graphics_display));
    let display = Display::register_shared(&shared_native_display)?;

    let mut screen = display.get_str_act()?;

    let mut screen_style = Style::default();
    screen_style.set_bg_color(State::DEFAULT, Color::from_rgb((255, 255, 255)));
    screen_style.set_radius(State::DEFAULT, 0);
    screen.add_style(Part::Main, &mut screen_style)?;

    // Create the arc object
    let mut arc = Arc::new()?;
    arc.set_size(150, 150)?;
    arc.set_align(&mut screen, Align::Center, 0, 10)?;
    arc.set_start_angle(135)?;
    arc.set_end_angle(135)?;

    let mut loading_lbl = Label::new()?;
    loading_lbl.set_text(CString::new("Loading...").unwrap().as_c_str())?;
    loading_lbl.set_align(&mut arc, Align::OutTopMid, 0, -10)?;
    loading_lbl.set_label_align(LabelAlign::Center)?;

    let mut loading_style = Style::default();
    loading_style.set_text_color(State::DEFAULT, Color::from_rgb((0, 0, 0)));
    loading_lbl.add_style(Part::Main, &mut loading_style)?;

    let mut angle = 0;
    let mut forward = true;
    let mut i = 0;

    // LVGL timer thread
    thread::spawn(|| {
        let interval = Duration::from_millis(5);
        loop {
            thread::sleep(interval);
            lvgl::tick_inc(interval);
        }
    });

    'running: loop {
        if i > 270 {
            forward = if forward { false } else { true };
            i = 1;
            println!("mem info running: {:?}", mem_info());
        }
        angle = if forward { angle + 1 } else { angle - 1 };
        arc.set_end_angle(angle + 135)?;
        i += 1;

        lvgl::task_handler();
        {
            let eg_display = shared_native_display.lock();
            window.update(&eg_display);
        }

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }
        thread::sleep(Duration::from_millis(15));
    }

    Ok(())
}
