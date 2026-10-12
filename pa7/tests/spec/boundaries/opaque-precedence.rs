macro_rules! m { ($e:expr) => {fn f()->i32{$e*3}} }
m!(1+2);
