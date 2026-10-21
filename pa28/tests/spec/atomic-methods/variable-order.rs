use core::sync::atomic::{AtomicU32,Ordering};fn f(a:&AtomicU32,o:Ordering)->u32{a.store(7,o);a.load(o)}
