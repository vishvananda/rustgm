mod provider {pub struct Wrapper {pub callback:fn()->u32} fn helper()->u32{7} pub fn make()->Wrapper{Wrapper{callback:helper}}}
fn answer()->u32{let value=provider::make();(value.callback)()}

fn failure(x:u32)->u32{1u32/x} fn main(){if answer()!=7 {let _=failure(0);}}
