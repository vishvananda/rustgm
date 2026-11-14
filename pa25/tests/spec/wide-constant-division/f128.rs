#![feature(f16,f128)]
const PI:f128=3.14159265358979323846264338327950288419716939937510582097494;const fn divide(x:f128,y:f128)->f128{x/y}
const V0:f128=1.0/3.0;
const V1:f128=180.0/PI;
const V2:f128=PI/180.0;
const V3:f128=-1.0/3.0;
const V4:f128=-0.0/3.0;
const V5:f128=-0.0/-3.0;
const V6:f128=1.0/0.0;
const V7:f128=-1.0/0.0;
const V8:f128=0.0/0.0;
const V9:f128=(1.0/0.0)/(1.0/0.0);
const V10:f128=(0.0/0.0)/1.0;
const V11:f128=1.0/(1.0/0.0);
const V12:f128=6e-4966/2.0;
const V13:f128=2e-4965/2.0;
const V14:f128=1e-4932/3.0;
const V15:f128=1e4932/0.5;
const V16:f128=divide(1.0,3.0);
const V17:f128=divide(180.0,PI);
#[unsafe(no_mangle)]pub static TABLE:[f128;18]=[V0,V1,V2,V3,V4,V5,V6,V7,V8,V9,V10,V11,V12,V13,V14,V15,V16,V17];
#[unsafe(no_mangle)]pub extern "C" fn address()->*const u8{&raw const TABLE as *const u8}
