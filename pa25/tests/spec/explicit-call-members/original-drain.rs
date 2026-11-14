#![feature(const_trait_impl,unboxed_closures,fn_traits)]
use core::marker::PhantomData;
pub struct Drain<'l,'f,T,F>{f:&'f mut F,l:PhantomData<&'l mut T>}
const impl<T, U, F> FnOnce<(usize,)> for &mut Drain<'_, '_, T, F>
where
    F: [const] FnMut(T) -> U,
{
    type Output = U;

    /// This implementation is useless.
    extern "rust-call" fn call_once(mut self, args: (usize,)) -> Self::Output {
        self.call_mut(args)
    }
}
const impl<T,U,F>FnMut<(usize,)> for &mut Drain<'_,'_,T,F>where F:[const]FnMut(T)->U{extern "rust-call" fn call_mut(&mut self,_:(usize,))->Self::Output{loop{}}}
fn main(){}
