use crate::lv_core::style::Style;
use crate::Box;
use crate::{Align, LvError, LvResult};
use core::ptr;

/// Represents a native LVGL object
pub trait NativeObject {
    /// Provide common way to access to the underlying native object pointer.
    fn raw(&self) -> LvResult<ptr::NonNull<lvgl_sys::lv_obj_t>>;
}

/// Generic LVGL object.
///
/// This is the parent object of all widget types. It stores the native LVGL raw pointer.
pub struct Obj {
    // We use a raw pointer here because we do not control this memory address, it is controlled
    // by LVGL's global state.
    raw: *mut lvgl_sys::lv_obj_t,
}

impl NativeObject for Obj {
    fn raw(&self) -> LvResult<ptr::NonNull<lvgl_sys::lv_obj_t>> {
        if let Some(non_null_ptr) = ptr::NonNull::new(self.raw) {
            Ok(non_null_ptr)
        } else {
            Err(LvError::InvalidReference)
        }
    }
}

/// A wrapper for all LVGL common operations on generic objects.
pub trait Widget: NativeObject {
    type SpecialEvent;
    type Part: Into<u8>;

    /// Construct an instance of the object from a raw pointer.
    ///
    fn from_raw(raw_pointer: ptr::NonNull<lvgl_sys::lv_obj_t>) -> Self;

    fn add_style(&self, part: Self::Part, style: &mut Style) -> LvResult<()> {
        unsafe {
            lvgl_sys::lv_obj_add_style(
                self.raw()?.as_mut(),
                part.into(),
                style.raw.as_mut() as *mut _,
            );
        };
        Ok(())
    }

    fn set_pos(&mut self, x: i16, y: i16) -> LvResult<()> {
        unsafe {
            lvgl_sys::lv_obj_set_pos(
                self.raw()?.as_mut(),
                x as lvgl_sys::lv_coord_t,
                y as lvgl_sys::lv_coord_t,
            );
        }
        Ok(())
    }

    fn set_size(&mut self, w: i16, h: i16) -> LvResult<()> {
        unsafe {
            lvgl_sys::lv_obj_set_size(
                self.raw()?.as_mut(),
                w as lvgl_sys::lv_coord_t,
                h as lvgl_sys::lv_coord_t,
            );
        }
        Ok(())
    }

    fn set_width(&mut self, w: u32) -> LvResult<()> {
        unsafe {
            lvgl_sys::lv_obj_set_width(self.raw()?.as_mut(), w as lvgl_sys::lv_coord_t);
        }
        Ok(())
    }

    fn set_height(&mut self, h: u32) -> LvResult<()> {
        unsafe {
            lvgl_sys::lv_obj_set_height(self.raw()?.as_mut(), h as lvgl_sys::lv_coord_t);
        }
        Ok(())
    }

    fn set_align<C>(&mut self, base: &mut C, align: Align, x_mod: i32, y_mod: i32) -> LvResult<()>
    where
        C: NativeObject,
    {
        unsafe {
            lvgl_sys::lv_obj_align(
                self.raw()?.as_mut(),
                base.raw()?.as_mut(),
                align.into(),
                x_mod as lvgl_sys::lv_coord_t,
                y_mod as lvgl_sys::lv_coord_t,
            );
        }
        Ok(())
    }
}

impl Widget for Obj {
    type SpecialEvent = ();
    type Part = Part;

    fn from_raw(raw: ptr::NonNull<lvgl_sys::lv_obj_t>) -> Self {
        Self { raw: raw.as_ptr() }
    }
}

impl Default for Obj {
    fn default() -> Self {
        Self {
            raw: unsafe { lvgl_sys::lv_obj_create(ptr::null_mut(), ptr::null_mut()) },
        }
    }
}

macro_rules! define_object {
    ($item:ident) => {
        define_object!($item, event = (), part = $crate::Part);
    };
    ($item:ident, event = $event_type:ty) => {
        define_object!($item, event = $event_type, part = $crate::Part);
    };
    ($item:ident, part = $part_type:ty) => {
        define_object!($item, event = (), part = $part_type);
    };
    ($item:ident, part = $part_type:ty, event = $event_type:ty) => {
        define_object!($item, event = $event_type, part = $part_type);
    };
    ($item:ident, event = $event_type:ty, part = $part_type:ty) => {
        pub struct $item {
            core: $crate::Obj,
        }

        unsafe impl Send for $item {}

        impl $item {
            pub fn on_event<F>(&mut self, f: F) -> $crate::LvResult<()>
            where
                F: FnMut(Self, $crate::support::Event<<Self as $crate::Widget>::SpecialEvent>),
            {
                use $crate::NativeObject;
                unsafe {
                    let mut raw = self.raw()?;
                    let obj = raw.as_mut();
                    let user_closure = $crate::Box::new(f);
                    obj.user_data = $crate::Box::into_raw(user_closure) as *mut cty::c_void;
                    lvgl_sys::lv_obj_set_event_cb(
                        obj,
                        lvgl_sys::lv_event_cb_t::Some($crate::support::event_callback::<Self, F>),
                    );
                }
                Ok(())
            }
        }

        impl $crate::NativeObject for $item {
            fn raw(&self) -> $crate::LvResult<core::ptr::NonNull<lvgl_sys::lv_obj_t>> {
                self.core.raw()
            }
        }

        impl $crate::Widget for $item {
            type SpecialEvent = $event_type;
            type Part = $part_type;

            fn from_raw(raw_pointer: core::ptr::NonNull<lvgl_sys::lv_obj_t>) -> Self {
                Self {
                    core: $crate::Obj::from_raw(raw_pointer),
                }
            }
        }
    };
}

// define_object!(Rafael);
//
// impl Rafael {
//     pub fn create(
//         parent: &mut impl crate::NativeObject,
//         copy: Option<&Rafael>,
//     ) -> crate::LvResult<Self> {
//         unsafe {
//             let ptr = lvgl_sys::lv_arc_create(
//                 parent.raw()?.as_mut(),
//                 copy.map(|c| c.raw().unwrap().as_mut() as *mut lvgl_sys::lv_obj_t)
//                     .unwrap_or(core::ptr::null_mut() as *mut lvgl_sys::lv_obj_t),
//             );
//             if let Some(raw) = core::ptr::NonNull::new(ptr) {
//                 let core = <crate::Obj as crate::Widget>::from_raw(raw);
//                 Ok(Self { core })
//             } else {
//                 Err(crate::LvError::InvalidReference)
//             }
//         }
//     }
//
//     pub fn create_at(parent: &mut impl crate::NativeObject) -> crate::LvResult<Self> {
//         Ok(Self::create(parent, None)?)
//     }
//
//     pub fn new() -> crate::LvResult<Self> {
//         let mut parent = crate::display::DefaultDisplay::get_scr_act()?;
//         Ok(Self::create_at(&mut parent)?)
//     }
// }

bitflags! {
    pub struct State: u32 {
        /// Normal, released
        const DEFAULT  = lvgl_sys::LV_STATE_DEFAULT;
        /// Toggled or checked
        const CHECKED  = lvgl_sys::LV_STATE_CHECKED;
        /// Focused via keypad or encoder or clicked via touchpad/mouse
        const FOCUSED  = lvgl_sys::LV_STATE_FOCUSED;
        /// Edit by an encoder
        const EDITED   = lvgl_sys::LV_STATE_EDITED;
        /// Hovered by mouse (not supported now)
        const HOVERED  = lvgl_sys::LV_STATE_HOVERED;
        /// Pressed
        const PRESSED  = lvgl_sys::LV_STATE_PRESSED;
        /// Disabled or inactive
        const DISABLED = lvgl_sys::LV_STATE_DISABLED;
    }
}

impl State {
    pub(crate) fn get_bits(&self) -> u32 {
        self.bits
    }
}

pub enum Part {
    Main,
    All,
}

impl Into<u8> for Part {
    fn into(self) -> u8 {
        match self {
            Part::Main => lvgl_sys::LV_OBJ_PART_MAIN as u8,
            Part::All => lvgl_sys::LV_OBJ_PART_ALL as u8,
        }
    }
}
