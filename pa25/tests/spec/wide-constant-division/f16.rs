#![feature(f16,f128)]
const PI:f16=3.14159265358979323846264338327950288419716939937510582097494;const fn divide(x:f16,y:f16)->f16{x/y}
const V0:f16=1.0/3.0;
const V1:f16=180.0/PI;
const V2:f16=PI/180.0;
const V3:f16=-1.0/3.0;
const V4:f16=-0.0/3.0;
const V5:f16=-0.0/-3.0;
const V6:f16=1.0/0.0;
const V7:f16=-1.0/0.0;
const V8:f16=0.0/0.0;
const V9:f16=(1.0/0.0)/(1.0/0.0);
const V10:f16=(0.0/0.0)/1.0;
const V11:f16=1.0/(1.0/0.0);
const V12:f16=5.9604644775390625e-8/2.0;
const V13:f16=1.78813934326171875e-7/2.0;
const V14:f16=0.00003/3.0;
const V15:f16=65504.0/0.5;
const V16:f16=divide(1.0,3.0);
const V17:f16=divide(180.0,PI);
#[unsafe(no_mangle)]pub static TABLE:[f16;18]=[V0,V1,V2,V3,V4,V5,V6,V7,V8,V9,V10,V11,V12,V13,V14,V15,V16,V17];
#[unsafe(no_mangle)]pub extern "C" fn address()->*const u8{&raw const TABLE as *const u8}
