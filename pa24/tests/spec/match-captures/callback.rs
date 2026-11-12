fn make(mut f:impl FnMut(u8)->Option<u8>)->impl FnMut(u8)->u8{move|item|match f(item){Some(x)=>x,None=>7}}
