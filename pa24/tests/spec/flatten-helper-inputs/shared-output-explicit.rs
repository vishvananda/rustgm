trait IntoIterator{type IntoIter;fn into_iter(self)->Self::IntoIter;}
trait Iterator:Sized{type Item;fn fold<B,F>(self,init:B,f:F)->B where F:FnMut(B,Self::Item)->B;}
fn flatten<T:IntoIterator,B>(fold:&mut impl FnMut(B,T::IntoIter)->B)->impl FnMut(B,T)->B+'_ {move|acc,item|fold(acc,item.into_iter())}
struct Left<T>(T);struct Right<T>(T);
impl<T> IntoIterator for Left<T>{type IntoIter=u8;fn into_iter(self)->u8{7}}
impl<T> IntoIterator for Right<T>{type IntoIter=u8;fn into_iter(self)->u8{9}}
fn code<X,I:Iterator<Item=Left<X>>,B,F:FnMut(B,u8)->B>(iter:I,acc:B,mut fold:F)->B{iter.fold(acc,flatten::<Left<X>,B>(&mut fold))}
