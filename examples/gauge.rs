use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl::style::{Opacity, Style};
use lvgl::widgets::Gauge;
use lvgl::{self, Align, Color, LvError, Part, State, Widget, UI};
use std::time::Instant;

fn main() -> Result<(), LvError> {
    let display: SimulatorDisplay<Rgb565> =
        SimulatorDisplay::new(Size::new(lvgl::HOR_RES_MAX, lvgl::VER_RES_MAX));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Gauge Example", &output_settings);

    let mut ui = UI::init()?;

    // Implement and register your display:
    ui.disp_drv_register(display)?;

    // Create screen and widgets
    let mut screen = ui.scr_act()?;

    let mut screen_style = Style::default();
    screen_style.set_bg_color(State::DEFAULT, Color::from_rgb((0, 0, 0)));
    screen.add_style(Part::Main, screen_style)?;

    // Create the gauge
    let mut gauge_style = Style::default();
    // Set a background color and a radius
    gauge_style.set_radius(State::DEFAULT, 5);
    gauge_style.set_bg_opa(State::DEFAULT, Opacity::OPA_COVER);
    gauge_style.set_bg_color(State::DEFAULT, Color::from_rgb((192, 192, 192)));
    // Set some padding's
    gauge_style.set_pad_inner(State::DEFAULT, 20);
    gauge_style.set_pad_top(State::DEFAULT, 20);
    gauge_style.set_pad_left(State::DEFAULT, 5);
    gauge_style.set_pad_right(State::DEFAULT, 5);

    gauge_style.set_scale_end_color(State::DEFAULT, Color::from_rgb((255, 0, 0)));
    gauge_style.set_line_color(State::DEFAULT, Color::from_rgb((255, 255, 255)));
    gauge_style.set_scale_grad_color(State::DEFAULT, Color::from_rgb((0, 0, 255)));
    gauge_style.set_line_width(State::DEFAULT, 2);
    gauge_style.set_scale_end_line_width(State::DEFAULT, 4);
    gauge_style.set_scale_end_border_width(State::DEFAULT, 4);

    let mut gauge = Gauge::new(&mut screen)?;
    gauge.add_style(Part::Main, gauge_style)?;
    gauge.set_align(&mut screen, Align::Center, 0, 0)?;
    gauge.set_value(0, 50)?;

    let mut i = 0;
    let mut loop_started = Instant::now();
    'running: loop {
        gauge.set_value(0, i)?;

        ui.task_handler();
        window.update(ui.get_display_ref().unwrap());

        for event in window.events() {
            match event {
                SimulatorEvent::MouseButtonUp {
                    mouse_btn: _,
                    point,
                } => {
                    println!("Clicked on: {:?}", point);
                }
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }

        if i > 99 {
            i = 0;
        } else {
            i = i + 1;
        }

        ui.tick_inc(loop_started.elapsed());
        loop_started = Instant::now();
    }

    Ok(())
}
