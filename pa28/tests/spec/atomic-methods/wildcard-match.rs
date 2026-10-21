use core::sync::atomic::{AtomicU32,Ordering};fn f(o:Ordering)->u32{match o{Ordering::Relaxed=>0,Ordering::Release=>1,Ordering::Acquire=>2,Ordering::AcqRel=>3,Ordering::SeqCst=>4,_=>5}}
