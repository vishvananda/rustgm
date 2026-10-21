use one::I8 as SIGNED;
const WIDTH:usize=two::WIDTH;
const VALUE:u8=one::U8;
enum E{A=one::USIZE as isize}
fn width<const N:usize>(_: [u8;N])->usize{N}
fn inverse(value:f32)->f32{1.0/value}
fn wide_inverse(value:f64)->f64{1.0/value}
fn unequal(value:f64)->bool{value!=value}
fn check(ok:bool){if !ok{let _=fail(0);}}
fn fail(n:u8)->u8{1/n}
fn main(){let _=one::UNIT;check(one::BOOL&&one::CHAR=='🦀'&&VALUE==7);check(one::U16==65535&&one::U32==4294967295&&one::U64==18446744073709551615&&one::U128==340282366920938463463374607431768211455&&one::USIZE==7);check(SIGNED == -1&&one::I16 == -1&&one::I32 == -1&&one::I64 == -1&&one::I128 == -170141183460469231731687303715884105728&&one::ISIZE == -1);check(match -1i8{two::SIGNED=>true,_=>false});check(width::<{two::WIDTH}>([1,2,3,4])==WIDTH&&WIDTH==4);check(E::A as isize==7);check(inverse(one::F32) == -inverse(one::ZERO));check(wide_inverse(two::NEGATIVE_ZERO) == -wide_inverse(0.0));check(unequal(one::NAN));let f:fn(u8)->u8=two::runtime;check(f(1)==9);}
