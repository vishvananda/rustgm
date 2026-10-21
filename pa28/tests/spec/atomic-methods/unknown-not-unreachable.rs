#![deny(unreachable_patterns)]use core::sync::atomic::{AtomicU32,Ordering};fn f(o:Ordering){match o{Ordering::Relaxed|Ordering::Release|Ordering::Acquire|Ordering::AcqRel|Ordering::SeqCst=>(),_=>()}}
