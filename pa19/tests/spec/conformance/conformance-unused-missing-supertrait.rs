fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{}trait B:A{}struct S;impl B for S{}fn main(){}
