use core::sync::atomic::{AtomicU32,Ordering};fn f(o:Option<Ordering>)->u32{match o{None=>0,Some(Ordering::Relaxed|Ordering::Release|Ordering::Acquire|Ordering::AcqRel|Ordering::SeqCst)=>1}}
