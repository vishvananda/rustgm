use core::mem::ManuallyDrop;
#[derive(Clone)]struct CloneOnly(u32);
#[derive(Clone,Copy)]union Generic<T>{value:ManuallyDrop<T>,word:u32}
pub fn rejected(){let value=Generic::<CloneOnly>{word:7};let _=value.clone();}
