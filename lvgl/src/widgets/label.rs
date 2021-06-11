use crate::widgets::Label;
use crate::{LvResult, NativeObject};

impl Label {
    pub fn set_label_align(&mut self, align: LabelAlign) -> LvResult<()> {
        unsafe {
            lvgl_sys::lv_label_set_align(self.core.raw()?.as_mut(), align as u8);
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
mod alloc_imp {
    use crate::widgets::Label;
    use crate::LvError;
    use cstr_core::CString;
    use core::convert::TryFrom;

    impl<S: AsRef<str>> From<S> for Label {
        fn from(text: S) -> Self {
            // text.try_into().unwrap()
            let text_cstr = CString::new(text.as_ref()).unwrap();
            let mut label = Label::new().unwrap();
            label.set_text(text_cstr.as_c_str()).unwrap();
            label
        }
    }

    // Issue link: https://github.com/rust-lang/rust/issues/50133
    //
    // impl<S: AsRef<str>> TryFrom<S> for Label {
    //     type Error = LvError;
    //     fn try_from(text: S) -> Result<Self, Self::Error> {
    //         let text_cstr = CString::new(text.as_ref())?;
    //         let mut label = Label::new()?;
    //         label.set_text(text_cstr.as_c_str())?;
    //         Ok(label)
    //     }
    // }
}


#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum LabelAlign {
    Left = lvgl_sys::LV_LABEL_ALIGN_LEFT as u8,
    Center = lvgl_sys::LV_LABEL_ALIGN_CENTER as u8,
    Right = lvgl_sys::LV_LABEL_ALIGN_RIGHT as u8,
    Auto = lvgl_sys::LV_LABEL_ALIGN_AUTO as u8,
}
