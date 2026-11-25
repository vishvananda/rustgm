#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}

#[lang = "deref"]
pub trait Deref: PointeeSized {
    #[lang = "deref_target"]
    type Target: PointeeSized;
    fn deref(&self) -> &Self::Target;
}

#[lang = "pin"]
#[fundamental]
#[repr(transparent)]
pub struct Pin<Ptr> {
    pub pointer: Ptr,
}

impl<T: PointeeSized> Deref for Pin<&mut T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.pointer
    }
}
impl<T: PointeeSized> LegacyReceiver for Pin<&mut T> {}

#[lang = "Context"]
pub struct Context<'a> {
    pub _marker: &'a (),
}

#[lang = "ResumeTy"]
pub struct ResumeTy(pub *mut Context<'static>);

#[cfg(rustc_oracle)]
#[lang = "get_context"]
pub unsafe fn get_context<'a, 'b>(_cx: ResumeTy) -> &'a mut Context<'b> {
    loop {}
}

#[lang = "Poll"]
pub enum Poll<T> {
    #[lang = "Ready"]
    Ready(T),
    #[lang = "Pending"]
    Pending,
}

#[lang = "future_trait"]
pub trait Future {
    #[lang = "future_output"]
    type Output;
    #[lang = "poll"]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

#[lang = "async_drop"]
pub trait AsyncDrop {
    #[allow(async_fn_in_trait)]
    async fn drop(self: Pin<&mut Self>);
}

#[lang = "async_drop_in_place"]
pub async unsafe fn async_drop_in_place<T: ?Sized>(_to_drop: &mut T) {}

pub trait AsyncWork {
    #[allow(async_fn_in_trait)]
    async fn run(&self) -> u32;
}

pub struct ReadyWork(pub u32);

pub fn check_value(v: ReadyWork) -> u32 {
    v.0
}

pub fn accept_async_drop<T: AsyncDrop + ?Sized>(_: &T) -> u32 {
    7
}

pub fn accept_async_work<T: AsyncWork>(_: &T) -> u32 {
    9
}
