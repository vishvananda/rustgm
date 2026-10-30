#![feature(rustc_attrs,stmt_expr_attributes)]
unsafe fn hidden()->u32{7}pub fn f(){let _=#[rustc_no_writable]||hidden();}
