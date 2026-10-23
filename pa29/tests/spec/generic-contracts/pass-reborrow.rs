fn fail(x:u8)->u8{1/x}fn main(){let n=7u8;let r=base::reborrow(&n);if *r!=7{let _=fail(0);}}
