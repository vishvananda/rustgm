struct S;trait Make{fn make<T:Copy>()->impl Copy;}impl Make for u8{fn make<T:Copy>()->[T;0]{[]}}fn main(){let a:[S;0]=<u8 as Make>::make();let _=a;}
