type Pair<T>=(T,T);
fn fill(x:&mut u8,y:&mut u8){
    let pair:Pair<_>=(x,y);
    *pair.1=19;
    *x=73;
}
fn main(){}
