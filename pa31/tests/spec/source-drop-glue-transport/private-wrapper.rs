// A fresh caller reaches the provider's private glue through an ordinary body.
pub unsafe fn cleanup<T: PointeeSized>(value: &mut T) {
    unsafe { drop_glue(&mut *value) }
}
