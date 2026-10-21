use core::sync::atomic::AtomicU32;
fn f()->&'static Option<AtomicU32>{&Some(AtomicU32::new(0))}
