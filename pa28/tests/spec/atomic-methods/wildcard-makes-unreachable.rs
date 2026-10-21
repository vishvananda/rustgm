#![deny(unreachable_patterns)]use core::sync::atomic::{AtomicU32,Ordering};fn f(o:Ordering){match o{_=>(),Ordering::Relaxed=>()}}
