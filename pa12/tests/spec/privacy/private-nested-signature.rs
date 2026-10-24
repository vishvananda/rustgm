mod provider {struct Hidden; pub fn take(_:fn()->(Hidden,)) {}}
fn answer()->u32{let _=provider::take;7}

fn failure(x:u32)->u32{1u32/x} fn main(){if answer()!=7 {let _=failure(0);}}
