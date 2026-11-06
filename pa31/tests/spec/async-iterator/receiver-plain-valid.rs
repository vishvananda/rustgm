pub struct Plain<'a>{value:&'a u8}
impl<'a>Plain<'a>{pub fn choose<'b>(self:Plain<'b>)->&'b u8 where 'a:'b,'b:'a{self.value}}
