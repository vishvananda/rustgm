fn make<'a>(value:&'a mut u8)->impl FnMut(bool)->u8+'a{move|flag|match flag{true=>{*value=7;*value},false=>*value}}
