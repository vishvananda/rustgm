use extension::{Number,Value};
#[inline(never)]fn dyn_value(value:&(dyn Value+'static))->u8{
    let call:fn(&(dyn Value+'static))->u8=<dyn Value>::read;
    call(value)
}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{extension::anchor();extension::make().read()}
#[unsafe(no_mangle)]pub extern "C" fn second()->u8{let value=Number(9);dyn_value(&value)}
