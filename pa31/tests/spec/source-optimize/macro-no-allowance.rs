macro_rules! make{($m:meta)=>{#[$m]pub fn f()->u32{7}}}make!(optimize(size));
