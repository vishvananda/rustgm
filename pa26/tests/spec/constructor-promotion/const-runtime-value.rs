const M:core::mem::ManuallyDrop<u32>=core::mem::ManuallyDrop::new(7);fn main(){let _=M;let _=core::mem::ManuallyDrop::new(9u32);}
