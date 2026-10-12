enum E{A,B(i32)}fn f(e:E)->i32{match e{E::A=>0,E::B(x)=>x}}
