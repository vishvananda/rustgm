fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Chooser(bool);
impl Chooser{fn pick<U>(&self,a:U,b:U)->U{if self.0{a}else{b}}}
fn get<'a>(c:&Chooser,x:&'a [u8])->&'a [u8]{c.pick(&[],x)}
fn main(){let a=[1u8,2];check(get(&Chooser(false),&a).len()==2);check(get(&Chooser(true),&a).len()==0);}
