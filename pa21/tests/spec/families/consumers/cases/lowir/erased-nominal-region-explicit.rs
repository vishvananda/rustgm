trait Q<'a,'b>{type Out where 'a:'b;}impl<'a,'b>Q<'a,'b>for u8{type Out=u8 where 'a:'b;}struct Hold<'a:'b,'b>(<u8 as Q<'a,'b>>::Out,&'a u8,&'b u8);
fn main(){}
