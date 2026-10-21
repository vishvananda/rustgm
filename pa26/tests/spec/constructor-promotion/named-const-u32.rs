const N:core::mem::ManuallyDrop<u32>=core::mem::ManuallyDrop::new(7);fn f()->&'static core::mem::ManuallyDrop<u32>{&N}fn main(){}
