struct S<T>(T);impl<T>S<T>{const C:u8={let n=7;n};}pub fn run()->bool{S::<u8>::C==7&&S::<u16>::C==7}
