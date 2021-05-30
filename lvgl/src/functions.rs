use crate::display::{Display, DisplayDriver};
use crate::{Color, Obj, Widget};
use core::ptr::NonNull;
use core::{ptr, result};
use embedded_graphics::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CoreError {
    ResourceNotAvailable,
    OperationFailed,
}

type Result<T> = result::Result<T, CoreError>;

/// Register own buffer
pub fn disp_drv_register<C: PixelColor + From<Color>, T: DrawTarget<C>>(
    disp_drv: &mut DisplayDriver<T, C>,
) -> Result<Display> {
    let disp_ptr = unsafe {
        lvgl_sys::lv_disp_drv_register(&mut disp_drv.disp_drv as *mut lvgl_sys::lv_disp_drv_t)
    };
    Ok(Display::from_raw(
        NonNull::new(disp_ptr).ok_or(CoreError::OperationFailed)?,
    ))
}

pub(crate) fn disp_get_default() -> Result<Display> {
    let disp_ptr = unsafe { lvgl_sys::lv_disp_get_default() };
    Ok(Display::from_raw(
        NonNull::new(disp_ptr).ok_or(CoreError::OperationFailed)?,
    ))
}

pub(crate) fn get_str_act(disp: Option<&Display>) -> Result<Obj> {
    let scr_ptr = unsafe {
        lvgl_sys::lv_disp_get_scr_act(
            disp.map(|d| d.disp.as_ptr())
                .unwrap_or(ptr::null_mut() as *mut lvgl_sys::lv_disp_t),
        )
    };
    Ok(Obj::from_raw(
        NonNull::new(scr_ptr).ok_or(CoreError::ResourceNotAvailable)?,
    ))
}
