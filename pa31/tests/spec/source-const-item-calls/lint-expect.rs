#![feature(rustc_attrs)]
#![expect(const_item_interior_mutations)]
pub struct S;
impl S{#[rustc_should_not_be_called_on_const_items]pub fn f(&self)->u32{7}}const C:S=S;pub fn observe()->u32{C.f()}
