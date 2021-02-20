use crate::{LvError, LvResult};
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

/// Places `T` into LVGL memory.
pub(crate) struct Box<T>(NonNull<T>);

impl<T> Box<T> {
    pub fn new(inner: T) -> LvResult<Box<T>> {
        let size = mem::size_of::<T>();
        let inner = unsafe {
            let ptr = lvgl_sys::lv_mem_alloc(size as lvgl_sys::size_t) as *mut T;

            // LVGL should align the memory address for us!
            assert_eq!(
                ptr as usize % mem::align_of::<T>(),
                0,
                "Memory address not aligned!"
            );

            match NonNull::new(ptr) {
                Some(v) => {
                    // Move `T` to LVGL managed memory
                    // It will panic if LVGL memory is not aligned
                    ptr.write(inner);
                    Ok(v)
                }
                None => Err(LvError::LvOOMemory),
            }
        };
        Ok(Box(inner?))
    }

    pub fn into_raw(self) -> *mut T {
        let b = mem::ManuallyDrop::new(self);
        b.0.as_ptr()
    }
}

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            lvgl_sys::lv_mem_free(self.0.as_ptr() as *const cty::c_void);
        }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> AsMut<T> for Box<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::mem::MaybeUninit;

    fn init() {
        unsafe {
            lvgl_sys::lv_init();
        };
    }

    fn teardown() {
        unsafe {
            lvgl_sys::lv_deinit();
        }
    }

    #[test]
    fn place_value_in_lv_mem() {
        init();

        let v = Box::new(5).unwrap();
        drop(v);
        let v = Box::new(10).unwrap();
        drop(v);

        teardown();
    }

    #[test]
    fn place_complex_value_in_lv_mem() {
        init();

        #[repr(C)]
        #[derive(Debug)]
        struct Point {
            x: u64,
            y: i8,
            t: i32,
            disp: i32,
        }

        for i in 0..100 {
            let p = Point {
                x: i,
                y: 42,
                t: 0,
                disp: -100,
            };

            println!("{:?}", p);
            let mut b = Box::new(p).unwrap_or_else(|_| {
                print_mem_info();
                panic!("OOM");
            });

            println!("memory address is {:p}", b.as_mut());

            let point = b.as_mut();
            if point.x != i {
                print_mem_info();

                println!("{:?}", point);
            }
            assert_eq!(point.x, i);
        }

        teardown();
    }

    fn print_mem_info() {
        let mut info = MaybeUninit::uninit();
        unsafe {
            lvgl_sys::lv_mem_monitor(info.as_mut_ptr());
        }
        if !info.as_ptr().is_null() {
            let info = unsafe { info.assume_init() };
            println!("mem info: {:?}", info);
        }
    }
}
