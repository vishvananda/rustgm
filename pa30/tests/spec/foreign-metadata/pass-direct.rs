fn main(){check(one::add(20,22)==42);check(two::relay(39)==42);check(one::implicit(40)==42);check(one::exact(38)==42);check(one::ordinary(39)==42);check(two::ordinary(39)==42);}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(n:u8)->u8{1/n}

