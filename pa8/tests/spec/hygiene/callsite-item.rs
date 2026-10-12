fn value()->i32{1} macro_rules! m{()=>{value()}} mod inner{fn value()->i32{2} fn f()->i32{m!()}}
