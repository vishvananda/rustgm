fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type Pair<T>=(T,T);
fn fill(x:&mut u8,y:&mut u8){
    let pair:Pair<_>=(x,y);
    *pair.0=17;
    *pair.1=19;
    *y=73;
}
fn main(){let mut x=0;let mut y=0;fill(&mut x,&mut y);check(x==17);check(y==73);}
