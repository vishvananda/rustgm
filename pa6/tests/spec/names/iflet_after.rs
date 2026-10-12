enum E{A(i32)}fn f(e:E){if let E::A(x)=e{x;}x;}
