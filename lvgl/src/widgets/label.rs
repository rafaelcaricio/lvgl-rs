use crate::widgets::Label;
use crate::{LvResult, NativeObject};

#[cfg(feature = "alloc")]
use cstr_core::CString;

impl Label {
    pub fn set_label_align(&mut self, align: LabelAlign) -> LvResult<()> {
        unsafe {
            lvgl_sys::lv_label_set_align(self.core.raw()?.as_mut(), align as u8);
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<S: AsRef<str>> From<S> for Label {
    fn from(text: S) -> Self {
        let text_cstr = CString::new(text.as_ref()).unwrap();
        let mut label = Label::new().unwrap();
        label.set_text(text_cstr.as_c_str()).unwrap();
        label
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum LabelAlign {
    Left = lvgl_sys::LV_LABEL_ALIGN_LEFT as u8,
    Center = lvgl_sys::LV_LABEL_ALIGN_CENTER as u8,
    Right = lvgl_sys::LV_LABEL_ALIGN_RIGHT as u8,
    Auto = lvgl_sys::LV_LABEL_ALIGN_AUTO as u8,
}
