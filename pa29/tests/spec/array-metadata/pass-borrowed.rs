fn fail(x:u8)->u8{1/x}fn main(){let a=[1u16,2,3,4];let b=arrays::borrowed(&a);if b[2]!=3{let _=fail(0);}}
