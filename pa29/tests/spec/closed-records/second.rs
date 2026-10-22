pub struct Pair { pub x:u32, pub y:u16 }
pub struct Nested { pub pair:Pair, pub tag:u8 }
struct Private { value:u8 }
pub struct Opaque { private:Private }
pub struct Borrowed { pub byte:&'static u8 }
static BYTE:u8=17;
fn helper(x:u32)->u32{x+1}
pub fn pair(x:u32)->Pair{Pair{x:helper(x),y:9}}
pub fn nested(x:u32)->Nested{Nested{pair:pair(x),tag:3}}
pub fn take(v:Pair)->u32{v.x+v.y as u32}
pub fn opaque(x:u8)->Opaque{Opaque{private:Private{value:x}}}
pub fn reveal(v:Opaque)->u8{v.private.value}
pub fn borrowed()->Borrowed{Borrowed{byte:&BYTE}}
