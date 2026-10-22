pub struct Marker{pub value:u32}
pub extern "C" fn relay(v:u32)->u32{one::add(v,3)}
pub extern "C" fn record_value(v:&Marker)->u32{v.value}
pub fn ordinary(v:u32)->u32{one::ordinary(v)}

