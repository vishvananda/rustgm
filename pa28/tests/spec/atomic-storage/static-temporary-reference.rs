use core::sync::atomic::{AtomicU32,Ordering};use core::mem::ManuallyDrop;
static R:&AtomicU32=&AtomicU32::new(0);
