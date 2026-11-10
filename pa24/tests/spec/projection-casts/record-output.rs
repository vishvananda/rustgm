fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read{type Output;fn read(self)->Self::Output;}
impl Read for usize{type Output=u8;fn read(self)->u8{self as u8}}
fn read<I:Read>(index:I)->I::Output{index.read()}
struct Record;impl Read for Record{type Output=Record;fn read(self)->Record{self}}fn main(){let f=|n|read(n) as usize;let _=f(Record);}
