fn make(mut f:impl FnMut(u8)->Option<u8>,mut test:impl FnMut(&u8)->bool)->impl FnMut(u8)->u8{move|item|match f(item){Some(x) if test(&x)=>x,_=>7}}
