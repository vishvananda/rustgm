trait Marker {}
impl Marker for u32 {}
trait Family {
    type Of<'x>;
}
struct Known;
impl Family for Known {
    type Of<'x> = u32;
}
trait Walk {}
trait FusedWalk: Walk {}
struct Pair<'a, T>(&'a T);
impl<'b, T: Family> Walk for Pair<'b, T> where <T as Family>::Of<'static>: Marker {}
impl<'a> FusedWalk for Pair<'a, Known> where <Known as Family>::Of<'a>: Marker {}

fn main() {}
