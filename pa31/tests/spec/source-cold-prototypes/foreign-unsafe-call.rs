unsafe extern "C"{#[cold]fn external(value:u8)->u8;}pub fn observed(v:u8)->u8{external(v)}
