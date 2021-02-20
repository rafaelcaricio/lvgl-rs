use crate::mem::Box;
use crate::LvResult;
use core::mem::MaybeUninit;
use embedded_graphics::geometry::Point;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum InputData {
    Touch(Point),
    Key(u32),
}

impl InputData {
    pub fn released(self) -> InputState {
        InputState::Released(self)
    }

    pub fn pressed(self) -> InputState {
        InputState::Pressed(self)
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum InputState {
    Released(InputData),
    Pressed(InputData),
}

impl InputState {
    pub fn once(self) -> BufferStatus {
        BufferStatus::Once(self)
    }

    pub fn and_continued(self) -> BufferStatus {
        BufferStatus::Buffered(self)
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BufferStatus {
    Once(InputState),
    Buffered(InputState),
}

pub struct Pointer {
    pub(crate) driver: lvgl_sys::lv_indev_drv_t,
    pub(crate) descriptor: Option<lvgl_sys::lv_indev_t>,
}

impl Pointer {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn() -> BufferStatus,
    {
        let driver = unsafe {
            let mut indev_drv = MaybeUninit::uninit();
            lvgl_sys::lv_indev_drv_init(indev_drv.as_mut_ptr());
            let mut indev_drv = indev_drv.assume_init();
            indev_drv.type_ = lvgl_sys::LV_INDEV_TYPE_POINTER as lvgl_sys::lv_indev_type_t;
            indev_drv.read_cb = Some(read_input::<F>);
            indev_drv.user_data = Box::into_raw(Box::new(handler).unwrap()) as *mut _
                as lvgl_sys::lv_indev_drv_user_data_t;
            indev_drv
        };
        Self {
            driver,
            descriptor: None,
        }
    }

    pub(crate) unsafe fn set_descriptor(
        &mut self,
        descriptor: *mut lvgl_sys::lv_indev_t,
    ) -> LvResult<()> {
        // TODO: check if not null && check if `self.descriptor` is not already set!
        self.descriptor = Some(*descriptor);
        Ok(())
    }
}

unsafe extern "C" fn read_input<F>(
    indev_drv: *mut lvgl_sys::lv_indev_drv_t,
    data: *mut lvgl_sys::lv_indev_data_t,
) -> bool
where
    F: Fn() -> BufferStatus,
{
    // convert user data to function
    let user_closure = &mut *((*indev_drv).user_data as *mut F);
    // call user data
    let info: BufferStatus = user_closure();
    match info {
        BufferStatus::Once(InputState::Pressed(InputData::Touch(point))) => {
            (*data).point.x = point.x as lvgl_sys::lv_coord_t;
            (*data).point.y = point.y as lvgl_sys::lv_coord_t;
            (*data).state = lvgl_sys::LV_INDEV_STATE_PR as lvgl_sys::lv_indev_state_t;
            false
        }
        BufferStatus::Once(InputState::Released(InputData::Touch(point))) => {
            (*data).point.x = point.x as lvgl_sys::lv_coord_t;
            (*data).point.y = point.y as lvgl_sys::lv_coord_t;
            (*data).state = lvgl_sys::LV_INDEV_STATE_REL as lvgl_sys::lv_indev_state_t;
            false
        }
        BufferStatus::Buffered(InputState::Pressed(InputData::Touch(point))) => {
            (*data).point.x = point.x as lvgl_sys::lv_coord_t;
            (*data).point.y = point.y as lvgl_sys::lv_coord_t;
            (*data).state = lvgl_sys::LV_INDEV_STATE_PR as lvgl_sys::lv_indev_state_t;
            true
        }
        BufferStatus::Buffered(InputState::Released(InputData::Touch(point))) => {
            (*data).point.x = point.x as lvgl_sys::lv_coord_t;
            (*data).point.y = point.y as lvgl_sys::lv_coord_t;
            (*data).state = lvgl_sys::LV_INDEV_STATE_REL as lvgl_sys::lv_indev_state_t;
            true
        }
        BufferStatus::Once(InputState::Released(InputData::Key(_))) => false,
        BufferStatus::Once(InputState::Pressed(InputData::Key(_))) => false,
        BufferStatus::Buffered(InputState::Released(InputData::Key(_))) => true,
        BufferStatus::Buffered(InputState::Pressed(InputData::Key(_))) => true,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::UI;
    use core::marker::PhantomData;
    use embedded_graphics::drawable::Pixel;
    use embedded_graphics::geometry::Size;
    use embedded_graphics::pixelcolor::PixelColor;
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics::DrawTarget;

    struct FakeDisplay<C>
    where
        C: PixelColor,
    {
        p: PhantomData<C>,
    }

    impl<C> DrawTarget<C> for FakeDisplay<C>
    where
        C: PixelColor,
    {
        type Error = ();

        fn draw_pixel(&mut self, item: Pixel<C>) -> Result<(), Self::Error> {
            Ok(())
        }

        fn size(&self) -> Size {
            Size::new(crate::VER_RES_MAX, crate::HOR_RES_MAX)
        }
    }

    //#[test]
    // We cannot test right now by having instances of UI global state... :(
    // I need to find a way to test while having global state...
    fn pointer_input_device() -> LvResult<()> {
        let mut ui = UI::init()?;

        let disp: FakeDisplay<Rgb565> = FakeDisplay { p: PhantomData };

        ui.disp_drv_register(disp)?;

        fn read_touchpad_device() -> BufferStatus {
            InputData::Touch(Point::new(120, 23)).pressed().once()
        }

        let mut touch_screen = Pointer::new(|| read_touchpad_device());

        ui.indev_drv_register(&mut touch_screen)?;

        Ok(())
    }
}
