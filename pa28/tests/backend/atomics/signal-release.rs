use std::sync::atomic::{AtomicUsize,Ordering::*,compiler_fence};fn main(){let a=AtomicUsize::new(0);a.store(19,Relaxed);compiler_fence(Release);std::process::exit(if a.load(Relaxed)==19{42}else{1})}
