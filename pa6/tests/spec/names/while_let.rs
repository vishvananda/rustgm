enum E{A(i32),B}fn f(e:E){while let E::A(x)=e{x;}}
