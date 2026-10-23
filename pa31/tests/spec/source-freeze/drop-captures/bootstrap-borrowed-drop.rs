struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0=7;}}#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{let mut count=0u8;{let _d=D(&mut count);}count}
