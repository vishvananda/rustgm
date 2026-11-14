const fn bad()->&'static u8{let x=7u8;unsafe{&*(&x as *const u8)}}static BAD:&u8=bad();
