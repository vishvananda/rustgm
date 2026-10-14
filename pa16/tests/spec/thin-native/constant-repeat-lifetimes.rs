fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Held<'a>(&'a u8);
const ITEM:Held<'static>=Held(&23);
fn pair<'a>(_:&'a u8)->[Held<'a>;2]{[ITEM;2]}
fn one<'a>(_:&'a u8)->[Held<'a>;1]{[(ITEM);1]}
fn empty<'a>(_:&'a u8)->[Held<'a>;0]{[ITEM;0]}
fn main(){let seed=7;let a=pair(&seed);check(*a[0].0==23 && *a[1].0==23);let b=one(&seed);check(*b[0].0==23);let c=empty(&seed);match c{[]=>()}}
