trait IntoIterator{type IntoIter;fn into_iter(self)->Self::IntoIter;}
trait Iterator:Sized{type Item;fn fold<B,F>(self,init:B,f:F)->B where F:FnMut(B,Self::Item)->B;}
fn flatten<T:IntoIterator,B>(fold:&mut impl FnMut(B,T::IntoIter)->B)->impl FnMut(B,T)->B+'_ {move|acc,item|fold(acc,item.into_iter())}
struct Left;struct Right;
impl IntoIterator for Left{type IntoIter=u8;fn into_iter(self)->u8{7}}
impl IntoIterator for Right{type IntoIter=u8;fn into_iter(self)->u8{9}}
fn code<B,F:FnMut(B,u8)->B>(fold:&mut F){let _wrapped=flatten(fold);}
