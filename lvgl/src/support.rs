use crate::Widget;
use core::convert::{TryFrom, TryInto};
use core::ptr::NonNull;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};

pub type LvResult<T> = Result<T, LvError>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum LvError {
    InvalidReference,
    Uninitialized,
    LvOOMemory,
    AlreadyInUse,
}

#[derive(Clone)]
pub struct Color {
    pub(crate) raw: lvgl_sys::lv_color_t,
}

impl Color {
    pub fn from_rgb((r, g, b): (u8, u8, u8)) -> Self {
        let raw = unsafe { lvgl_sys::_LV_COLOR_MAKE(r, g, b) };
        Self { raw }
    }

    pub fn from_raw(raw: lvgl_sys::lv_color_t) -> Self {
        Self { raw }
    }

    pub fn r(&self) -> u8 {
        unsafe {
            lvgl_sys::_LV_COLOR_GET_R(self.raw) as u8
        }
    }

    pub fn g(&self) -> u8 {
        unsafe {
            lvgl_sys::_LV_COLOR_GET_G(self.raw) as u8
        }
    }

    pub fn b(&self) -> u8 {
        unsafe {
            lvgl_sys::_LV_COLOR_GET_B(self.raw) as u8
        }
    }
}

impl From<Color> for Rgb888 {
    fn from(color: Color) -> Self {
        unsafe {
            Rgb888::new(
                lvgl_sys::_LV_COLOR_GET_R(color.raw) as u8,
                lvgl_sys::_LV_COLOR_GET_G(color.raw) as u8,
                lvgl_sys::_LV_COLOR_GET_B(color.raw) as u8,
            )
        }
    }
}

impl From<Color> for Rgb565 {
    fn from(color: Color) -> Self {
        unsafe {
            Rgb565::new(
                lvgl_sys::_LV_COLOR_GET_R(color.raw) as u8,
                lvgl_sys::_LV_COLOR_GET_G(color.raw) as u8,
                lvgl_sys::_LV_COLOR_GET_B(color.raw) as u8,
            )
        }
    }
}

/// Events are triggered in LittlevGL when something happens which might be interesting to
/// the user, e.g. if an object:
///  - is clicked
///  - is dragged
///  - its value has changed, etc.
///
/// All objects (such as Buttons/Labels/Sliders etc.) receive these generic events
/// regardless of their type.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Event<T> {
    /// The object has been pressed
    Pressed,

    /// The object is being pressed (sent continuously while pressing)
    Pressing,

    /// The input device is still being pressed but is no longer on the object
    PressLost,

    /// Released before `long_press_time` config time. Not called if dragged.
    ShortClicked,

    /// Called on release if not dragged (regardless to long press)
    Clicked,

    /// Pressing for `long_press_time` config time. Not called if dragged.
    LongPressed,

    /// Called after `long_press_time` config in every `long_press_rep_time` ms. Not
    /// called if dragged.
    LongPressedRepeat,

    /// Called in every case when the object has been released even if it was dragged. Not called
    /// if slid from the object while pressing and released outside of the object. In this
    /// case, `Event<_>::PressLost` is sent.
    Released,

    /// Pointer-like input devices events (E.g. mouse or touchpad)
    Pointer(PointerEvent),

    /// Special event for the object type
    Special(T),
}

impl<S> TryFrom<lvgl_sys::lv_event_t> for Event<S> {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value as u32 {
            lvgl_sys::LV_EVENT_PRESSED => Ok(Event::Pressed),
            lvgl_sys::LV_EVENT_PRESSING => Ok(Event::Pressing),
            lvgl_sys::LV_EVENT_PRESS_LOST => Ok(Event::PressLost),
            lvgl_sys::LV_EVENT_SHORT_CLICKED => Ok(Event::ShortClicked),
            lvgl_sys::LV_EVENT_CLICKED => Ok(Event::Clicked),
            lvgl_sys::LV_EVENT_LONG_PRESSED => Ok(Event::LongPressed),
            lvgl_sys::LV_EVENT_LONG_PRESSED_REPEAT => Ok(Event::LongPressedRepeat),
            lvgl_sys::LV_EVENT_RELEASED => Ok(Event::Released),
            _ => Err(()),
            // _ => {
            //     if let Ok(special_event_type) = S::try_from(value) {
            //         Ok(Event::Special(special_event_type))
            //     } else {
            //         Err(())
            //     }
            // }
        }
    }
}

impl<S> From<Event<S>> for lvgl_sys::lv_event_t {
    fn from(event: Event<S>) -> Self {
        let native_event = match event {
            Event::Pressed => lvgl_sys::LV_EVENT_PRESSED,
            Event::Pressing => lvgl_sys::LV_EVENT_PRESSING,
            Event::PressLost => lvgl_sys::LV_EVENT_PRESS_LOST,
            Event::ShortClicked => lvgl_sys::LV_EVENT_SHORT_CLICKED,
            Event::Clicked => lvgl_sys::LV_EVENT_CLICKED,
            Event::LongPressed => lvgl_sys::LV_EVENT_LONG_PRESSED,
            Event::LongPressedRepeat => lvgl_sys::LV_EVENT_LONG_PRESSED_REPEAT,
            Event::Released => lvgl_sys::LV_EVENT_RELEASED,
            // TODO: handle all types...
            _ => lvgl_sys::LV_EVENT_CLICKED,
        };
        native_event as lvgl_sys::lv_event_t
    }
}

/// These events are sent only by pointer-like input devices (E.g. mouse or touchpad)
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PointerEvent {
    DragBegin,
    DragEnd,
    DragThrowBegin,
}

pub(crate) unsafe extern "C" fn event_callback<T, F>(
    obj: *mut lvgl_sys::lv_obj_t,
    event: lvgl_sys::lv_event_t,
) where
    T: Widget + Sized,
    F: FnMut(T, Event<T::SpecialEvent>),
{
    // convert the lv_event_t to lvgl-rs Event type
    if let Ok(event) = event.try_into() {
        if let Some(obj_ptr) = NonNull::new(obj) {
            let object = T::from_raw(obj_ptr);
            // get the pointer from the Rust callback closure FnMut provided by users
            let user_closure = &mut *((*obj).user_data as *mut F);
            // call user callback closure
            user_closure(object, event);
        }
    }
}

pub enum Align {
    Center,
    InTopLeft,
    InTopMid,
    InTopRight,
    InBottomLeft,
    InBottomMid,
    InBottomRight,
    InLeftMid,
    InRightMid,
    OutTopLeft,
    OutTopMid,
    OutTopRight,
    OutBottomLeft,
    OutBottomMid,
    OutBottomRight,
    OutLeftTop,
    OutLeftMid,
    OutLeftBottom,
    OutRightTop,
    OutRightMid,
    OutRightBottom,
}

impl Into<u8> for Align {
    fn into(self) -> u8 {
        let native = match self {
            Align::Center => lvgl_sys::LV_ALIGN_CENTER,
            Align::InTopLeft => lvgl_sys::LV_ALIGN_IN_TOP_LEFT,
            Align::InTopMid => lvgl_sys::LV_ALIGN_IN_TOP_MID,
            Align::InTopRight => lvgl_sys::LV_ALIGN_IN_TOP_RIGHT,
            Align::InBottomLeft => lvgl_sys::LV_ALIGN_IN_BOTTOM_LEFT,
            Align::InBottomMid => lvgl_sys::LV_ALIGN_IN_BOTTOM_MID,
            Align::InBottomRight => lvgl_sys::LV_ALIGN_IN_BOTTOM_RIGHT,
            Align::InLeftMid => lvgl_sys::LV_ALIGN_IN_LEFT_MID,
            Align::InRightMid => lvgl_sys::LV_ALIGN_IN_RIGHT_MID,
            Align::OutTopLeft => lvgl_sys::LV_ALIGN_OUT_TOP_LEFT,
            Align::OutTopMid => lvgl_sys::LV_ALIGN_OUT_TOP_MID,
            Align::OutTopRight => lvgl_sys::LV_ALIGN_OUT_TOP_RIGHT,
            Align::OutBottomLeft => lvgl_sys::LV_ALIGN_OUT_BOTTOM_LEFT,
            Align::OutBottomMid => lvgl_sys::LV_ALIGN_OUT_BOTTOM_MID,
            Align::OutBottomRight => lvgl_sys::LV_ALIGN_OUT_BOTTOM_RIGHT,
            Align::OutLeftTop => lvgl_sys::LV_ALIGN_OUT_LEFT_TOP,
            Align::OutLeftMid => lvgl_sys::LV_ALIGN_OUT_LEFT_MID,
            Align::OutLeftBottom => lvgl_sys::LV_ALIGN_OUT_LEFT_BOTTOM,
            Align::OutRightTop => lvgl_sys::LV_ALIGN_OUT_RIGHT_TOP,
            Align::OutRightMid => lvgl_sys::LV_ALIGN_OUT_RIGHT_MID,
            Align::OutRightBottom => lvgl_sys::LV_ALIGN_OUT_RIGHT_BOTTOM,
        };
        native as u8
    }
}

pub enum Animation {
    ON,
    OFF,
}

impl From<Animation> for lvgl_sys::lv_anim_enable_t {
    fn from(anim: Animation) -> Self {
        match anim {
            Animation::ON => lvgl_sys::LV_ANIM_ON as u8,
            Animation::OFF => lvgl_sys::LV_ANIM_OFF as u8,
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use lvgl_sys;

    #[test]
    fn color_properties_accessible() {
        let color = Color::from_rgb((206, 51, 255));

        if lvgl_sys::LV_COLOR_DEPTH == 32 {
            assert_eq!(color.r(), 206);
            assert_eq!(color.g(), 51);
            assert_eq!(color.b(), 255);
        } else if lvgl_sys::LV_COLOR_DEPTH == 16 {
            assert_eq!(color.r(), 25);
            assert_eq!(color.g(), 12);
            assert_eq!(color.b(), 31);
        }
    }
}
