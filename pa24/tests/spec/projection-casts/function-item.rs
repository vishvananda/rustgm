fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read{type Output;fn read(self)->Self::Output;}
struct Wrap<T>(T);impl<T> Read for Wrap<T>{type Output=T;fn read(self)->T{self.0}}
fn read<I:Read>(index:I)->I::Output{index.read()}
fn target()->u8{7}
fn main(){let cast=|x|read(x) as usize;let address=cast(Wrap(target));check(address==target as usize);}
