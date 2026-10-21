use core::sync::atomic::AtomicU32;
fn f()->&'static [AtomicU32;0]{&[const{AtomicU32::new(0)};0]}
