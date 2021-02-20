use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use lvgl::widgets::Keyboard;
use lvgl::LvError;
use lvgl::UI;
use std::time::Instant;

fn main() -> Result<(), LvError> {
    let display: SimulatorDisplay<Rgb565> =
        SimulatorDisplay::new(Size::new(lvgl::HOR_RES_MAX, lvgl::VER_RES_MAX));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Simple Example", &output_settings);

    // Initialize LVGL
    let mut ui = UI::init()?;

    // Register your display
    ui.disp_drv_register(display)?;

    // Get the active screen
    let mut screen = ui.scr_act()?;

    // Create a Keyboard widget on the screen
    let _ = Keyboard::new(&mut screen)?;

    let mut loop_started = Instant::now();
    'running: loop {
        // Tell LVGL to process UI related tasks
        ui.task_handler();

        // Update your window with the latest display image
        window.update(ui.get_display_ref().unwrap());

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }

        // Tell LVGL how much time has past since last loop
        ui.tick_inc(loop_started.elapsed());

        loop_started = Instant::now();
    }

    Ok(())
}
