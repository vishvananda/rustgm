#[macro_export] macro_rules! helper{()=>{42}} #[macro_export(local_inner_macros)] macro_rules! m{()=>{helper!()}} mod inner{fn f()->i32{crate::m!()}}
