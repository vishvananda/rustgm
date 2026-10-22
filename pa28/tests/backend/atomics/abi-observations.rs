use std::mem::{size_of,align_of};use std::sync::atomic::*;fn main(){
println!("i8 {} {} {} {}",size_of::<i8>(),align_of::<i8>(),size_of::<AtomicI8>(),align_of::<AtomicI8>());
println!("u8 {} {} {} {}",size_of::<u8>(),align_of::<u8>(),size_of::<AtomicU8>(),align_of::<AtomicU8>());
println!("i16 {} {} {} {}",size_of::<i16>(),align_of::<i16>(),size_of::<AtomicI16>(),align_of::<AtomicI16>());
println!("u16 {} {} {} {}",size_of::<u16>(),align_of::<u16>(),size_of::<AtomicU16>(),align_of::<AtomicU16>());
println!("i32 {} {} {} {}",size_of::<i32>(),align_of::<i32>(),size_of::<AtomicI32>(),align_of::<AtomicI32>());
println!("u32 {} {} {} {}",size_of::<u32>(),align_of::<u32>(),size_of::<AtomicU32>(),align_of::<AtomicU32>());
println!("i64 {} {} {} {}",size_of::<i64>(),align_of::<i64>(),size_of::<AtomicI64>(),align_of::<AtomicI64>());
println!("u64 {} {} {} {}",size_of::<u64>(),align_of::<u64>(),size_of::<AtomicU64>(),align_of::<AtomicU64>());
println!("isize {} {} {} {}",size_of::<isize>(),align_of::<isize>(),size_of::<AtomicIsize>(),align_of::<AtomicIsize>());
println!("usize {} {} {} {}",size_of::<usize>(),align_of::<usize>(),size_of::<AtomicUsize>(),align_of::<AtomicUsize>());
println!("bool {} {} {} {}",size_of::<bool>(),align_of::<bool>(),size_of::<AtomicBool>(),align_of::<AtomicBool>());
println!("ptr {} {} {} {}",size_of::<*mut u64>(),align_of::<*mut u64>(),size_of::<AtomicPtr<u64>>(),align_of::<AtomicPtr<u64>>());
}
