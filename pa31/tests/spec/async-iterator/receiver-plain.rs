pub struct Plain<'a>{value:&'a u8}
impl<'a>Plain<'a>{pub fn choose<'b>(self:Plain<'b>)->&'b u8{self.value}}
