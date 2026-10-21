union U{a:[u32;2],x:u32}fn f(u:&U)->*const u32{&raw const u.a[0]}fn main(){}
