// Original macro body from Rust 1.98 revision 88d9e12a core/src/ptr/mod.rs.
// Upstream attribution and license: NOTICE and third_party/sources.json.
#![allow(dead_code, unused_assignments)]
unsafe fn swap_chunk<const N:usize>(_:&mut[u8;N],_:&mut[u8;N]){}
fn swap_nonoverlapping_short(x:*mut u8,y:*mut u8,bytes:usize){
let mut i=0;
        macro_rules! swap_prefix {
            ($($n:literal)+) => {$(
                if (bytes & $n) != 0 {
                    // SAFETY: `i` can only have the same bits set as those in bytes,
                    // so these `add`s are in-bounds of `bytes`.  But the bit for
                    // `$n` hasn't been set yet, so the `$n` bytes that `swap_chunk`
                    // will read and write are within the usable range.
                    unsafe { swap_chunk::<$n>(&mut*x.add(i).cast(), &mut*y.add(i).cast()) };
                    i |= $n;
                }
            )+};
        }
        swap_prefix!(4 2 1);
}
fn main(){}
