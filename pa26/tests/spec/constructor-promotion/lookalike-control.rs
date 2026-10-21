struct ManuallyDrop(u32);fn f()->&'static ManuallyDrop{&ManuallyDrop(7)}fn main(){let _=f();}
