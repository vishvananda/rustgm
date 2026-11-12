fn make(mut f:impl FnMut(u8)->Option<u8>)->impl FnMut(u8)->bool{move|item|match f(item){Some(x)=>x,None=>7}}
