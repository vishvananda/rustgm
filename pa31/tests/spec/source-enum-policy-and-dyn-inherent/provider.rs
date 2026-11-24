#[lang="freeze"]pub unsafe auto trait Freeze{}
#[unstable(feature="unsize",issue="18598")]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[unstable(feature="coerce_unsized",issue="18598")]
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
#[unstable(feature="dispatch_from_dyn",issue="none")]
#[lang="dispatch_from_dyn"]pub trait DispatchFromDyn<T>:Sized{}
#[unstable(feature="dispatch_from_dyn",issue="none")]
impl<'a,T:PointeeSized+Unsize<U>,U:PointeeSized>DispatchFromDyn<&'a U> for &'a T{}
#[unstable(feature="coerce_unsized",issue="18598")]
impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized>CoerceUnsized<&'a U> for &'b T{}
#[rustc_diagnostic_item="Send"]
pub unsafe auto trait Send{}

pub enum Void{}

#[must_use="this outcome should be handled"]
pub enum Outcome<T,E>{
    Ok(T),
    Err(E),
}

pub fn ok_value(value:u8)->Outcome<u8,Void>{
    Outcome::Ok(value)
}

pub trait Marker{
    fn id(&self)->u8;
}

impl dyn Marker{
    pub fn tag(&self)->u8{self.id()}
}

impl dyn Marker+Send{
    pub fn shared_tag(&self)->u8{23}
}
