#[lang = "sync"]
pub unsafe auto trait Sync {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}
#[lang = "unsize"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Unsize<T: PointeeSized>: PointeeSized {}
#[lang = "coerce_unsized"]
pub trait CoerceUnsized<T: PointeeSized>: Sized {}
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'a T {}
impl<T: PointeeSized> Clone for &T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for &T {}
impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}

#[repr(transparent)]
pub struct Header(pub u32);
impl Clone for Header {
    fn clone(&self) -> Self {
        Header(self.0)
    }
}
impl Copy for Header {}

pub static HEADERS: [Header; 2] = [Header(19), Header(23)];

static DIGITS: &[u8; 4] = b"0123";

pub fn digits() -> &'static [u8; 4] {
    DIGITS
}

pub struct Entry {
    pub code: u16,
    pub delta: u8,
}

struct Table {
    entries: &'static [Entry],
}

static TABLE: Table = Table {
    entries: &[Entry { code: 7, delta: 11 }, Entry { code: 13, delta: 29 }],
};

pub fn table() -> &'static [Entry] {
    TABLE.entries
}

pub struct VTable {
    pub clone: fn() -> &'static VTable,
    pub tag: fn() -> u8,
}

pub struct Handle {
    pub vtable: &'static VTable,
}

pub const fn handle() -> Handle {
    static VTABLE: VTable = VTable {
        clone: || &VTABLE,
        tag: || 37,
    };
    Handle { vtable: &VTABLE }
}
