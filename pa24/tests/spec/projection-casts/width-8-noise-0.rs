fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read{type Output;fn read(self)->Self::Output;}
impl Read for usize{type Output=u8;fn read(self)->u8{self as u8}}
fn read<I:Read>(index:I)->I::Output{index.read()}
fn main(){let f0=|n|read(n) as usize;check(f0(7usize)==7);let f1=|n|read(n) as usize;check(f1(7usize)==7);let f2=|n|read(n) as usize;check(f2(7usize)==7);let f3=|n|read(n) as usize;check(f3(7usize)==7);let f4=|n|read(n) as usize;check(f4(7usize)==7);let f5=|n|read(n) as usize;check(f5(7usize)==7);let f6=|n|read(n) as usize;check(f6(7usize)==7);let f7=|n|read(n) as usize;check(f7(7usize)==7);}
