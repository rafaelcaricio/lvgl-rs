use cstr_core::CString;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl::style::Style;
use lvgl::widgets::{Arc, Label, LabelAlign};
use lvgl::{self, Align, Color, Part, State, UI};
use lvgl::{LvError, Widget};
use std::time::Instant;

fn main() -> Result<(), LvError> {
    let display: SimulatorDisplay<Rgb565> =
        SimulatorDisplay::new(Size::new(lvgl::HOR_RES_MAX, lvgl::VER_RES_MAX));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Arc Example", &output_settings);

    let mut ui = UI::init()?;

    // Implement and register your display:
    ui.disp_drv_register(display)?;

    // Create screen and widgets
    let mut screen = ui.scr_act()?;

    let mut screen_style = Style::default();
    screen_style.set_bg_color(State::DEFAULT, Color::from_rgb((255, 255, 255)));
    screen_style.set_radius(State::DEFAULT, 0);
    screen.add_style(Part::Main, screen_style)?;

    // Create the arc object
    let mut arc = Arc::new(&mut screen)?;
    arc.set_size(150, 150)?;
    arc.set_align(&mut screen, Align::Center, 0, 10)?;
    arc.set_start_angle(135)?;
    arc.set_end_angle(135)?;

    let mut loading_lbl = Label::new(&mut screen)?;
    loading_lbl.set_text(CString::new("Loading...").unwrap().as_c_str())?;
    loading_lbl.set_align(&mut arc, Align::OutTopMid, 0, -10)?;
    loading_lbl.set_label_align(LabelAlign::Center)?;

    let mut loading_style = Style::default();
    loading_style.set_text_color(State::DEFAULT, Color::from_rgb((0, 0, 0)));
    loading_lbl.add_style(Part::Main, loading_style)?;

    let mut angle = 0;
    let mut forward = true;
    let mut i = 0;

    let mut loop_started = Instant::now();
    'running: loop {
        if i > 270 {
            forward = if forward { false } else { true };
            i = 1;
        }
        angle = if forward { angle + 1 } else { angle - 1 };
        arc.set_end_angle(angle + 135)?;
        i += 1;

        ui.task_handler();
        window.update(ui.get_display_ref().unwrap());

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }

        ui.tick_inc(loop_started.elapsed());
        loop_started = Instant::now();
    }

    Ok(())
}
