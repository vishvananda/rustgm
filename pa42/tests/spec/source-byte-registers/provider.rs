impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}
impl Clone for i32 { fn clone(&self) -> Self { *self } }
impl Copy for i32 {}
impl Clone for i64 { fn clone(&self) -> Self { *self } }
impl Copy for i64 {}
impl<T> Clone for *const T { fn clone(&self) -> Self { *self } }
impl<T> Copy for *const T {}

pub mod word32 {
    use crate::arch::asm;
    // ORIGINAL_bt_macro_x86 END
    // ORIGINAL__bittest END
    // ORIGINAL__bittestandset END
    // ORIGINAL__bittestandreset END
    // ORIGINAL__bittestandcomplement END
}

pub mod word64 {
    use crate::arch::asm;
    // ORIGINAL_bt_macro_x86_64 END
    // ORIGINAL__bittest64 END
    // ORIGINAL__bittestandset64 END
    // ORIGINAL__bittestandreset64 END
    // ORIGINAL__bittestandcomplement64 END
}
