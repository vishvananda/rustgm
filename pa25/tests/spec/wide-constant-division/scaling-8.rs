#![feature(f128)]
const PI:f128=3.14159265358979323846264338327950288419716939937510582097494;
const V0:f128=180.0/PI;
const V1:f128=1e4932/6e-4966;
const V2:f128=6e-4966/1e4932;
const V3:f128=2e-4965/2.0;
const V4:f128=180.0/PI;
const V5:f128=1e4932/6e-4966;
const V6:f128=6e-4966/1e4932;
const V7:f128=2e-4965/2.0;
#[unsafe(no_mangle)]pub static TABLE:[f128;8]=[V0,V1,V2,V3,V4,V5,V6,V7];
#[unsafe(no_mangle)]pub extern "C" fn address()->*const u8{&raw const TABLE as *const u8}
