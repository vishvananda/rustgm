fn f<'a>()where &'a u8:'static{}fn g<'b:'static>(){f::<'b>();}
