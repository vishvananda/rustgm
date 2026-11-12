trait IntoIterator{type IntoIter;fn into_iter(self)->Self::IntoIter;}
trait Iterator:Sized{type Item;fn take(self)->Self::Item;fn fold<B,F>(self,init:B,mut fold:F)->B where F:FnMut(B,Self::Item)->B{fold(init,self.take())}}
fn flatten<T:IntoIterator,B>(fold:&mut impl FnMut(B,T::IntoIter)->B)->impl FnMut(B,T)->B+'_ {move|acc,item|fold(acc,item.into_iter())}
unsafe extern "C" { fn record(tag:u8,value:u8); }
struct Bucket(u8);struct Left(u8);struct Right(u8);struct One(Left);struct Two(Right);
impl IntoIterator for Left{type IntoIter=Bucket;fn into_iter(self)->Bucket{unsafe{record(1,self.0)};Bucket(self.0)}}
impl IntoIterator for Right{type IntoIter=Bucket;fn into_iter(self)->Bucket{unsafe{record(3,9)};Bucket(9)}}
impl Iterator for One{type Item=Left;fn take(self)->Left{self.0}}
impl Iterator for Two{type Item=Right;fn take(self)->Right{self.0}}
fn apply<I:Iterator,B,F>(iter:I,acc:B,mut fold:F)->B where I::Item:IntoIterator,F:FnMut(B,<I::Item as IntoIterator>::IntoIter)->B{iter.fold(acc,flatten(&mut fold))}
#[unsafe(no_mangle)]pub extern "C" fn code(value:u8)->u8{
 let first=apply(One(Left(value)),3,|_acc,bucket:Bucket|{unsafe{record(2,bucket.0)};bucket.0});
 let second=apply(Two(Right(value)),9,|_acc,bucket:Bucket|{unsafe{record(4,bucket.0)};bucket.0});
 first;second
}
