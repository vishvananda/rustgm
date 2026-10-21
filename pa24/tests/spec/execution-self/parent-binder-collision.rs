fn infer<'a,F:for<'b>Fn(&'b u8)->&'a u8>(f:F)->F{f}fn outer<'a>(x:&'a u8)->&'a u8{let f=infer(|v|v);f(x)}fn main(){}
