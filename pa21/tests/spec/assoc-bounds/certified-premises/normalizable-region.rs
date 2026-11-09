trait Marker {}
impl Marker for u32 {}
trait Family {
    type Of<'a>;
}
struct Known;
impl Family for Known {
    type Of<'a> = u32;
}
trait Walk {}
trait FusedWalk: Walk {}
struct Pair<'a, 'b>(&'a (), &'b ());
impl<'a, 'b> Walk for Pair<'a, 'b> where <Known as Family>::Of<'b>: Marker {}
impl<'a, 'b> FusedWalk for Pair<'a, 'b> where <Known as Family>::Of<'a>: Marker {}

fn main() {}
