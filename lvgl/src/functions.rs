use crate::display::{Display, DisplayDriver};
use crate::{Obj, Widget};
use core::ptr::NonNull;
use core::time::Duration;
use core::{ptr, result};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CoreError {
    ResourceNotAvailable,
    OperationFailed,
}

type Result<T> = result::Result<T, CoreError>;

/// Register own buffer
pub(crate) fn disp_drv_register(disp_drv: &mut DisplayDriver) -> Result<Display> {
    let disp_ptr = unsafe { lvgl_sys::lv_disp_drv_register(&mut disp_drv.disp_drv as *mut _) };
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

/// You have to call this function periodically.
/// Expects a `tick_period` duration as argument which is the call period of this
/// function in milliseconds.
#[inline]
pub fn tick_inc(tick_period: Duration) {
    unsafe {
        lvgl_sys::lv_tick_inc(tick_period.as_millis() as u32);
    }
}

/// Call it periodically to handle tasks.
#[inline]
pub fn task_handler() {
    unsafe { lvgl_sys::lv_task_handler() };
}
