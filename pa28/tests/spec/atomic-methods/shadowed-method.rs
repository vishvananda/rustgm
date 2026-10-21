struct AtomicU32;impl AtomicU32{fn load(&self,o:u8)->u32{o as u32}}fn f(a:&AtomicU32)->u32{a.load(1)}
