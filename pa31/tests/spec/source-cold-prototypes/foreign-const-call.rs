unsafe extern "C"{#[cold]fn external(value:u8)->u8;}const VALUE:u8=unsafe{external(23)};
