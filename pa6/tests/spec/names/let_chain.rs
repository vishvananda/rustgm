enum E{A(i32),B}fn f(e:E){if let E::A(x)=e && x>0{x;}}
