fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type Triple<T>=(T,(T,T));
fn fill(x:&mut u8,y:&mut u8,z:&mut u8){
    let triple:Triple<_>=(x,({y},{z}));
    *triple.0=17;
    *triple.1.0=19;
    *triple.1.1=23;
    *y=73;
    *z=79;
}
fn main(){let mut x=0;let mut y=0;let mut z=0;fill(&mut x,&mut y,&mut z);check(x==17);check(y==73);check(z==79);}
