pub struct Plain<'a>{value:&'a u8}
impl<'a>Plain<'a>{pub fn require_static(self:Plain<'static>)->&'static u8{self.value}}
fn escape(value:&u8)->&'static u8{Plain{value}.require_static()}
