pub struct Node { pub value:u32, pub next:&'static Node }
pub static NODE:Node=Node{value:41,next:&NODE};
