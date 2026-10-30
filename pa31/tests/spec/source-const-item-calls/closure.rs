#![feature(rustc_attrs,stmt_expr_attributes)]
pub fn observe()->u32{let f=#[rustc_should_not_be_called_on_const_items]||7;f()}
