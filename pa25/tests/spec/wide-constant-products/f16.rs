#![feature(f16,f128)]
const MAX:f16=6.5504e+4_f16;
const fn multiply(x:f16,y:f16)->f16{x*y}
const fn limit()->f16{const HI:f16=MAX * 0.5;HI}
const V0:f16=MAX*0.5;
const V1:f16=1.5*1.5;
const V2:f16=1.0009765625*1.0009765625;
const V3:f16=1.0009765625*1.5;
const V4:f16=1.0029296875*1.5;
const V5:f16=-0.0*3.0;
const V6:f16=-0.0*-3.0;
const V7:f16=(1.0/0.0)*-2.0;
const V8:f16=(1.0/0.0)*0.0;
const V9:f16=(0.0/0.0)*2.0;
const V10:f16=1.0*(1.0/0.0);
const V11:f16=5.9604644775390625e-8*0.5;
const V12:f16=1.78813934326171875e-7*0.5;
const V13:f16=0.00003*0.25;
const V14:f16=65504.0*65504.0;
const V15:f16=0.00003*0.00003;
const V16:f16=multiply(MAX,0.5);
const V17:f16=limit();
const V18:f16=1.9990234375*1.9990234375;
const V19:f16=0.0*(1.0/0.0);
const V20:f16=(1.0/0.0)*(-1.0/0.0);
const V21:f16=-5.9604644775390625e-8*0.25;
const V22:f16=MAX*5.9604644775390625e-8;
const V23:f16=MAX*0.00006103515625;
#[unsafe(no_mangle)]pub static TABLE:[f16;24]=[V0,V1,V2,V3,V4,V5,V6,V7,V8,V9,V10,V11,V12,V13,V14,V15,V16,V17,V18,V19,V20,V21,V22,V23];
#[unsafe(no_mangle)]pub extern "C" fn address()->*const u8{&raw const TABLE as *const u8}
