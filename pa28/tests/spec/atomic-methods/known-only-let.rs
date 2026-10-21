use core::sync::atomic::{AtomicU32,Ordering};fn f(o:Ordering){let (Ordering::Relaxed|Ordering::Release|Ordering::Acquire|Ordering::AcqRel|Ordering::SeqCst)=o;}
