fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Chooser(bool);
impl Chooser{fn pick<U>(&self,a:U,b:U)->U{if self.0{a}else{b}}}
fn main(){let a=[1u8,2];let c=Chooser(true);let r=c.pick(&[],&a);check(r.len()==0);}
