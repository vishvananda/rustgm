use core::sync::atomic::{AtomicU32,Ordering};fn f(o:Option<Ordering>)->u32{match o{None=>0,Some(_)=>1}}
