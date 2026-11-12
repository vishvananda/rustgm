trait IntoIterator{type IntoIter;fn into_iter(self)->Self::IntoIter;}
trait Iterator:Sized{type Item;fn fold<B,F>(self,init:B,f:F)->B where F:FnMut(B,Self::Item)->B;}
fn flatten<T:IntoIterator,B>(fold:&mut impl FnMut(B,T::IntoIter)->B)->impl FnMut(B,T)->B+'_ {move|acc,item|fold(acc,item.into_iter())}
fn code<I:Iterator+Copy,B,F>(iter:I,mut acc:B,mut fold:F)->B where I::Item:IntoIterator,F:FnMut(B,<I::Item as IntoIterator>::IntoIter)->B{acc=iter.fold(acc,flatten(&mut fold));acc=iter.fold(acc,flatten(&mut fold));acc=iter.fold(acc,flatten(&mut fold));acc=iter.fold(acc,flatten(&mut fold));acc=iter.fold(acc,flatten(&mut fold));acc=iter.fold(acc,flatten(&mut fold));acc=iter.fold(acc,flatten(&mut fold));acc=iter.fold(acc,flatten(&mut fold));acc}
