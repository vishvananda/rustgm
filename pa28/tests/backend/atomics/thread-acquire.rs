use std::sync::atomic::{AtomicUsize,Ordering::*,fence};fn main(){let a=AtomicUsize::new(0);a.store(19,Relaxed);fence(Acquire);std::process::exit(if a.load(Relaxed)==19{42}else{1})}
