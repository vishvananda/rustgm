#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[rustc_const_unstable(feature="offset_transport",issue="none")]
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[rustc_const_unstable(feature="offset_transport",issue="none")]
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[rustc_const_unstable(feature="offset_transport",issue="none")]
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}



#[lang = "freeze"]
pub unsafe auto trait Freeze {}
// ORIGINAL_OFFSET
#[repr(C)]
pub struct Fields {
    pub first: u8,
    pub middle: u64,
    pub last: u16,
}

#[rustc_const_unstable(feature="offset_transport",issue="none")]
pub const fn field_offset<T: PointeeSized>(ordinal: u32) -> usize {
    offset_of::<T>(0, ordinal)
}
