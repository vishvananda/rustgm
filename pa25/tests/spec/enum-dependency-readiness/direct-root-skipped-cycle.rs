fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=if false{F::B as isize}else{1}}enum F{B=E::A as isize}fn main(){}