#![feature(rustc_attrs,stmt_expr_attributes)]
pub fn observe()->u32{let f=#[rustc_confusables("swap")]||7;f()}
