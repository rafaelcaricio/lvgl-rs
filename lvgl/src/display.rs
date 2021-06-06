use crate::functions::CoreError;
use crate::{disp_drv_register, disp_get_default, get_str_act};
use crate::{Box, RunOnce};
use crate::{Color, Obj};
use core::cell::RefCell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use core::{ptr, result};
use parking_lot::const_mutex;
use parking_lot::Mutex;

pub const DISP_HOR_RES: usize = lvgl_sys::LV_HOR_RES_MAX as usize;
pub const DISP_VER_RES: usize = lvgl_sys::LV_VER_RES_MAX as usize;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DisplayError {
    NotAvailable,
    FailedToRegister,
    NotRegistered,
}

type Result<T> = result::Result<T, DisplayError>;

pub struct Display {
    pub(crate) disp: NonNull<lvgl_sys::lv_disp_t>,
}

impl Display {
    pub(crate) fn from_raw(disp: NonNull<lvgl_sys::lv_disp_t>) -> Self {
        Self { disp }
    }

    pub fn register<F, const N: usize>(
        draw_buffer: &'static DrawBuffer<N>,
        display_update: F,
    ) -> Result<Self>
    where
        F: FnMut(&DisplayRefresh<N>) + 'static,
    {
        let mut display_diver = DisplayDriver::new(draw_buffer, display_update)?;
        Ok(disp_drv_register(&mut display_diver)?)
    }

    pub fn get_scr_act(&self) -> Result<Obj> {
        Ok(get_str_act(Some(&self))?)
    }
}

impl Default for Display {
    fn default() -> Self {
        disp_get_default().expect("LVGL must be INITIALIZED")
    }
}

#[derive(Copy, Clone)]
pub struct DefaultDisplay {}

impl DefaultDisplay {
    /// Gets the screen active of the default display.
    pub fn get_scr_act() -> Result<Obj> {
        Ok(get_str_act(None)?)
    }
}

pub struct DrawBuffer<const N: usize> {
    initialized: RunOnce,
    refresh_buffer: Mutex<RefCell<[MaybeUninit<lvgl_sys::lv_color_t>; N]>>,
}

impl<const N: usize> DrawBuffer<N> {
    pub const fn new() -> Self {
        Self {
            initialized: RunOnce::new(),
            refresh_buffer: const_mutex(RefCell::new([MaybeUninit::uninit(); N])),
        }
    }

    fn get_ptr(&self) -> Option<Box<lvgl_sys::lv_disp_buf_t>> {
        if self.initialized.swap_and_check() {
            // TODO: needs to be 'static somehow
            // Cannot be in the DrawBuffer struct because the type `lv_disp_buf_t` contains a raw
            // pointer and raw pointers are not Sync and consequently cannot be in `static` variables.
            let mut inner: MaybeUninit<lvgl_sys::lv_disp_buf_t> = MaybeUninit::uninit();
            let primary_buffer_guard = self.refresh_buffer.lock();
            let draw_buf = unsafe {
                lvgl_sys::lv_disp_buf_init(
                    inner.as_mut_ptr(),
                    primary_buffer_guard.borrow_mut().as_mut_ptr() as *mut _ as *mut cty::c_void,
                    ptr::null_mut(),
                    N as u32,
                );
                inner.assume_init()
            };
            Some(Box::new(draw_buf))
        } else {
            None
        }
    }
}

pub struct DisplayDriver {
    pub(crate) disp_drv: lvgl_sys::lv_disp_drv_t,
}

impl DisplayDriver {
    pub fn new<F, const N: usize>(
        draw_buffer: &'static DrawBuffer<N>,
        display_update_callback: F,
    ) -> Result<Self>
    where
        F: FnMut(&DisplayRefresh<N>) + 'static,
    {
        let mut disp_drv = unsafe {
            let mut inner = MaybeUninit::uninit();
            lvgl_sys::lv_disp_drv_init(inner.as_mut_ptr());
            inner.assume_init()
        };

        // Safety: The variable `draw_buffer` is statically allocated, no need to worry about this being dropped.
        disp_drv.buffer = draw_buffer
            .get_ptr()
            .map(|ptr| Box::into_raw(ptr) as *mut _)
            .ok_or(DisplayError::FailedToRegister)?;

        disp_drv.user_data = Box::into_raw(Box::new(display_update_callback)) as *mut _
            as lvgl_sys::lv_disp_drv_user_data_t;

        // Sets trampoline pointer to the function implementation that uses the `F` type for a
        // refresh buffer of size N specifically.
        disp_drv.flush_cb = Some(disp_flush_trampoline::<F, N>);

        // We do not store any memory that can be accidentally deallocated by on the Rust side.
        Ok(Self { disp_drv })
    }
}

/// Represents a sub-area of the display that is being updated.
pub struct Area {
    pub x1: i16,
    pub x2: i16,
    pub y1: i16,
    pub y2: i16,
}

/// It's a update to the display information, contains the area that is being updated and the color
/// of the pixels that need to be updated. The colors are represented in a contiguous array.
pub struct DisplayRefresh<const N: usize> {
    pub area: Area,
    pub colors: [Color; N],
}

#[cfg(feature = "embedded_graphics")]
mod embedded_graphics_impl {
    use crate::{Color, DisplayRefresh};
    use embedded_graphics::drawable;
    use embedded_graphics::prelude::*;

    impl<const N: usize> DisplayRefresh<N> {
        pub fn as_pixels<C>(&self) -> impl IntoIterator<Item = drawable::Pixel<C>> + '_
        where
            C: PixelColor + From<Color>,
        {
            let area = &self.area;
            let x1 = area.x1;
            let x2 = area.x2;
            let y1 = area.y1;
            let y2 = area.y2;

            let ys = y1..=y2;
            let xs = (x1..=x2).enumerate();
            let x_len = (x2 - x1 + 1) as usize;

            // We use iterators here to ensure that the Rust compiler can apply all possible
            // optimizations at compile time.
            ys.enumerate()
                .map(move |(iy, y)| {
                    xs.clone().map(move |(ix, x)| {
                        let color_len = x_len * iy + ix;
                        let raw_color = self.colors[color_len];
                        drawable::Pixel(Point::new(x as i32, y as i32), raw_color.into())
                    })
                })
                .flatten()
        }
    }
}

unsafe extern "C" fn disp_flush_trampoline<F, const N: usize>(
    disp_drv: *mut lvgl_sys::lv_disp_drv_t,
    area: *const lvgl_sys::lv_area_t,
    color_p: *mut lvgl_sys::lv_color_t,
) where
    F: FnMut(&DisplayRefresh<N>) + 'static,
{
    let display_driver = *disp_drv;
    if !display_driver.user_data.is_null() {
        let callback = &mut *(display_driver.user_data as *mut F);

        let mut colors = [Color::default(); N];
        let mut color_len = 0;
        for color in &mut colors {
            let lv_color = *color_p.add(color_len);
            *color = Color::from_raw(lv_color);
            color_len += 1;
        }

        let update = DisplayRefresh {
            area: Area {
                x1: (*area).x1,
                x2: (*area).x2,
                y1: (*area).y1,
                y2: (*area).y2,
            },
            colors,
        };
        callback(&update);
    }

    // Indicate to LVGL that we are ready with the flushing
    lvgl_sys::lv_disp_flush_ready(disp_drv);
}

impl From<CoreError> for DisplayError {
    fn from(err: CoreError) -> Self {
        use DisplayError::*;
        match err {
            CoreError::ResourceNotAvailable => NotAvailable,
            CoreError::OperationFailed => NotAvailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests;

    #[test]
    fn get_scr_act_return_display() {
        tests::initialize_test();
        let _screen = get_str_act(None).expect("We can get the active screen");
    }

    #[test]
    fn get_default_display() {
        tests::initialize_test();
        let display = Display::default();

        let _screen_direct = display
            .get_scr_act()
            .expect("Return screen directly from the display instance");

        let _screen_default =
            DefaultDisplay::get_scr_act().expect("Return screen from the default display");
    }

    #[test]
    fn register_display_directly() -> Result<()> {
        tests::initialize_test();
        let display = Display::default();

        let _screen = display
            .get_scr_act()
            .expect("Return screen directly from the display instance");

        Ok(())
    }
}
