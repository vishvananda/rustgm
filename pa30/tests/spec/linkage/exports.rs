#[unsafe(no_mangle)]
extern "C" fn ffi_named(x:i32)->i32{x+3}
#[unsafe(export_name="ffi_renamed")]
extern "C" fn private_name(x:i32)->i32{x+3}
macro_rules! make{()=>{#[unsafe(export_name="ffi_macro")]extern "C" fn from_macro(x:i32)->i32{x+3}}}
make!();
#[cfg_attr(all(),unsafe(export_name="ffi_cfg"))]
extern "C" fn configured(x:i32)->i32{x+3}
#[unsafe(export_name="ffi.$0")]
extern "C" fn punctuation(x:i32)->i32{x+3}
#[unsafe(export_name="ffi_float")]
extern "C" fn floating(x:f64)->f64{x+2.5}
#[unsafe(export_name="ffi_self")]
extern "C" fn self_export(x:i32)->i32{x+3}
unsafe extern "C"{
    safe fn call_named(x:i32)->i32;
    safe fn call_renamed(x:i32)->i32;
    safe fn call_macro(x:i32)->i32;
    safe fn call_cfg(x:i32)->i32;
    safe fn call_special(x:i32)->i32;
    safe fn call_float(x:f64)->f64;
    #[link_name="ffi_self"]safe fn self_alias(x:i32)->i32;
}
fn check(ok:bool){if !ok{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
fn main(){check(call_named(39)==42);check(call_renamed(39)==42);check(call_macro(39)==42);check(call_cfg(39)==42);check(call_special(39)==42);check(call_float(1.25)==3.75);check(self_alias(39)==42);}
