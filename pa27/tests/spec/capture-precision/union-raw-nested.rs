union U{a:(u32,u32),b:u64}fn main(){let u=U{a:(7,9)};let c=||unsafe{&raw const u.a.0};let _=c();}
