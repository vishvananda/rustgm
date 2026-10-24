mod provider {struct Hidden {pub value:u32} pub fn make()->Hidden{Hidden{value:7}}}
fn answer()->u32{provider::make().value}

fn failure(x:u32)->u32{1u32/x} fn main(){if answer()!=7 {let _=failure(0);}}
