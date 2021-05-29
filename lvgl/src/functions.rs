use crate::display::{Display, DisplayDriver, DisplayError};
use crate::Color;
use core::ptr::NonNull;
use embedded_graphics::drawable;
use embedded_graphics::prelude::*;

pub fn disp_drv_register<C: PixelColor + From<Color>, T: DrawTarget<C>>(
    disp_drv: &mut DisplayDriver<T, C>,
) -> Result<Display, DisplayError> {
    let disp_ptr = unsafe {
        lvgl_sys::lv_disp_drv_register(&mut disp_drv.disp_drv as *mut lvgl_sys::lv_disp_drv_t)
    };
    Ok(Display::from_raw(
        NonNull::new(disp_ptr).ok_or(DisplayError::FailedToRegister)?,
    ))
}
