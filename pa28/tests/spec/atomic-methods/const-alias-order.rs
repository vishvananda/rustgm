use core::sync::atomic::{AtomicU32,Ordering};const ORDER:Ordering=Ordering::Release;fn f(a:&AtomicU32)->u32{a.load(ORDER)}
