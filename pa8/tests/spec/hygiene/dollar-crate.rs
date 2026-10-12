fn value()->i32{1} macro_rules! m{()=>{$crate::value()}} mod inner{fn f()->i32{m!()}}
