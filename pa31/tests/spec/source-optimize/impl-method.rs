#![feature(optimize_attribute)]
trait T{fn f()->u32;}impl T for (){#[optimize(none)]fn f()->u32{7}}
