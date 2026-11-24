#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[lang = "unsafe_cell"]
#[repr(transparent)]
pub struct Slot<T: ?Sized> {
    pub value: T,
}

impl<T: PointeeSized> !Freeze for Slot<T> {}

#[lang = "unsafe_pinned"]
#[repr(transparent)]
pub struct Pinned<T: ?Sized> {
    pub value: Slot<T>,
}

#[lang = "maybe_dangling"]
#[repr(transparent)]
pub struct Relax<T: ?Sized>(pub T);

#[lang = "manually_drop"]
#[repr(transparent)]
pub struct Hold<T: ?Sized>(pub Relax<T>);

#[repr(transparent)]
pub struct Ordinary<T: ?Sized>(pub T);
