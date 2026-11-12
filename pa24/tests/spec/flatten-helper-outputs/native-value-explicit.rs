trait IntoIterator{type IntoIter;fn into_iter(self)->Self::IntoIter;}
trait Iterator:Sized{type Item;fn take(self)->Self::Item;fn try_fold<B,F,R>(self,init:B,mut f:F)->R where F:FnMut(B,Self::Item)->R{f(init,self.take())}}
fn flatten<T:IntoIterator,B,R>(fold:&mut impl FnMut(B,T::IntoIter)->R)->impl FnMut(B,T)->R+'_ {move|acc,item|fold(acc,item.into_iter())}
unsafe extern "C" { fn record(tag:u8,value:u8); }
struct Bucket(u8);struct Left(u8);struct Right(u8);struct One(Left);struct Two(Right);
impl IntoIterator for Left{type IntoIter=Bucket;fn into_iter(self)->Bucket{unsafe{record(1,self.0)};Bucket(self.0)}}
impl IntoIterator for Right{type IntoIter=Bucket;fn into_iter(self)->Bucket{unsafe{record(3,9)};Bucket(9)}}
impl Iterator for One{type Item=Left;fn take(self)->Left{self.0}}
impl Iterator for Two{type Item=Right;fn take(self)->Right{self.0}}
fn apply<I:Iterator,B,R,F>(iter:I,acc:B,mut fold:F)->R where I::Item:IntoIterator,F:FnMut(B,<I::Item as IntoIterator>::IntoIter)->R{iter.try_fold(acc,flatten::<I::Item,B,R>(&mut fold))}
#[unsafe(no_mangle)]pub extern "C" fn code(value:u8)->u8{
 let first:u16=apply(One(Left(value)),3u8,|acc:u8,bucket:Bucket|{unsafe{record(2,acc)};bucket.0 as u16});
 let second:u8=apply(Two(Right(value)),9u8,|acc:u8,bucket:Bucket|{unsafe{record(4,acc)};bucket.0});
 if first==7{second}else{0}
}
