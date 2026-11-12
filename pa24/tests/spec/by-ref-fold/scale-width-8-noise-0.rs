struct Wrap<T>(T);
trait Item{type Output;}
impl<T>Wrap<T>{fn many<A0,A1,A2,A3,A4,A5,A6,A7,F>(mut f:F)->impl FnMut(A0,A1,A2,A3,A4,A5,A6,A7)->Self where F:FnMut(A0,A1,A2,A3,A4,A5,A6,A7)->T{move|a0,a1,a2,a3,a4,a5,a6,a7|Self(f(a0,a1,a2,a3,a4,a5,a6,a7))}}
fn code<I0:Item,I1:Item,I2:Item,I3:Item,I4:Item,I5:Item,I6:Item,I7:Item,B,F>(f:F)where F:FnMut(I0::Output,I1::Output,I2::Output,I3::Output,I4::Output,I5::Output,I6::Output,I7::Output)->B{let _=Wrap::many(f);}
