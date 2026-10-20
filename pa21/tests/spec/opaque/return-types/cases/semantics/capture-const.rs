fn f<const N:usize>()->impl Copy{[7u8;N]}fn main(){let _=f::<3>();let _=f::<4>();}
