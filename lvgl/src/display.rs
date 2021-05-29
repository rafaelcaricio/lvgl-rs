use crate::{Color, LvError, LvResult, Obj};
use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ptr;
use core::ptr::NonNull;
use embedded_graphics::drawable;
use embedded_graphics::prelude::*;

// TODO: Make this an external configuration
const REFRESH_BUFFER_LEN: usize = 2;
// Declare a buffer for the refresh rate
pub(crate) const BUF_SIZE: usize = lvgl_sys::LV_HOR_RES_MAX as usize * REFRESH_BUFFER_LEN;

pub enum DisplayError {
    FailedToRegister,
    NotRegistered,
}

#[derive(Copy, Clone)]
pub struct Display {
    disp: NonNull<lvgl_sys::lv_disp_t>,
}

impl Display {
    pub(crate) fn from_raw(disp: NonNull<lvgl_sys::lv_disp_t>) -> Self {
        Self { disp }
    }
}

#[derive(Copy, Clone)]
pub struct DefaultDisplay {}

impl DefaultDisplay {
    pub fn get_screen_active() -> Result<Obj, DisplayError> {
        Err(DisplayError::NotRegistered)
    }
}

#[derive(Copy, Clone)]
pub struct DisplayBuffer {
    disp_buf: lvgl_sys::lv_disp_buf_t,
}

impl DisplayBuffer {
    pub fn new() -> Self {
        let disp_buf = unsafe {
            let mut disp_buf = MaybeUninit::uninit();
            let mut refresh_buffer = ManuallyDrop::new([lvgl_sys::lv_color_t::default(); BUF_SIZE]);

            lvgl_sys::lv_disp_buf_init(
                disp_buf.as_mut_ptr(),
                refresh_buffer.as_mut_ptr() as *mut cty::c_void,
                ptr::null_mut(),
                lvgl_sys::LV_HOR_RES_MAX * REFRESH_BUFFER_LEN as u32,
            );
            disp_buf.assume_init()
        };

        Self { disp_buf }
    }
}

#[derive(Copy, Clone)]
pub struct DisplayDriver<C>
where
    C: PixelColor + From<Color>,
{
    pub(crate) disp_drv: lvgl_sys::lv_disp_drv_t,
    phantom: PhantomData<C>,
}

impl<C> DisplayDriver<C>
where
    C: PixelColor + From<Color>,
{
    pub fn new<I, F>(display_buffer: DisplayBuffer, callback: F) -> Self
    where
        I: IntoIterator<Item = drawable::Pixel<C>>,
        F: FnMut(I) + 'static,
    {
        let mut disp_buf = ManuallyDrop::new(display_buffer.disp_buf);
        let mut callback = ManuallyDrop::new(callback);
        let mut disp_drv = unsafe {
            let mut disp_drv = MaybeUninit::uninit();
            lvgl_sys::lv_disp_drv_init(disp_drv.as_mut_ptr());
            disp_drv.assume_init()
        };
        disp_drv.buffer = &mut *disp_buf as *mut lvgl_sys::lv_disp_buf_t;
        disp_drv.user_data = &mut callback as *mut _ as lvgl_sys::lv_disp_drv_user_data_t;
        disp_drv.flush_cb = Some(disp_flush_trampoline::<C, I, F>);

        Self {
            disp_drv,
            phantom: PhantomData,
        }
    }
}

// impl Default for DisplayDriver {
//     fn default() -> Self {
//         Self::new(DisplayBuffer::new())
//     }
// }

pub struct DisplayDriverBuilder {
    disp_buf: Option<DisplayBuffer>,
}

impl DisplayDriverBuilder {
    pub fn with_callback() {}
}

unsafe extern "C" fn disp_flush_trampoline<C, I, F>(
    disp_drv: *mut lvgl_sys::lv_disp_drv_t,
    area: *const lvgl_sys::lv_area_t,
    color_p: *mut lvgl_sys::lv_color_t,
) where
    C: PixelColor + From<Color>,
    I: IntoIterator<Item = drawable::Pixel<C>>,
    F: FnMut(I) + 'static,
{
    let display_driver = *disp_drv;
    if !display_driver.user_data.is_null() {
        let callback = &mut *(display_driver.user_data as *mut F);
        let x1 = (*area).x1;
        let x2 = (*area).x2;
        let y1 = (*area).y1;
        let y2 = (*area).y2;

        let ys = y1..=y2;
        let xs = (x1..=x2).enumerate();
        let x_len = (x2 - x1 + 1) as usize;

        // We use iterators here to ensure that the Rust compiler can apply all possible
        // optimizations at compile time.
        let pixels = ys
            .enumerate()
            .map(|(iy, y)| {
                xs.clone().map(move |(ix, x)| {
                    let color_len = x_len * iy + ix;
                    let lv_color = unsafe { *color_p.add(color_len) };
                    let raw_color = Color::from_raw(lv_color);
                    drawable::Pixel(Point::new(x as i32, y as i32), raw_color.into())
                })
            })
            .flatten();

        callback(pixels);
    }
}
