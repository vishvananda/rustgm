#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#[link(name="selected_native",kind="static")]unsafe extern "C"{fn selected()->u8;}
unsafe extern "C"{fn still_visible()->u8;}
unsafe extern "C"{fn observe(a:u8,b:u8);}
fn main(){unsafe{observe(selected(),still_visible());}}
