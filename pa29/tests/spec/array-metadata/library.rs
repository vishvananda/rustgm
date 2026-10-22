pub struct Token{pub n:u8}
pub struct Bundle{pub items:[Token;2]}
pub struct Grid{pub cells:[[u16;2];2]}
pub fn round(mut value:[u16;4])->[u16;4]{value[2]+=1;value}
pub fn nested(value:Grid)->u16{value.cells[1][1]}
pub fn borrowed(value:&[u16;4])->&[u16;4]{value}
pub fn empty()->[u8;0]{[]}
