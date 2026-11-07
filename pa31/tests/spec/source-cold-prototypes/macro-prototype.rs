macro_rules!method{()=>{#[cold]fn prototype()->u8;};}trait Owner{method!();}
