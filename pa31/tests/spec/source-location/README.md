This planned source lane exercises the selected pinned `Location` declaration,
its actual `Copy, Clone` derive list, the source `TrivialClone` marker and real
field `AssertParamIsClone` obligations. The provider reuses the independently
qualified reduced NonNull/Pointee and PhantomData foundations. The complete
upstream Pointee bounds, stability inventory, caller tracking, PanicInfo Debug,
formatting and false-assert path remain required; this is not their replacement.

The authored `observed` helper constructs the private representation using an
actual NUL-backed filename allocation and the preserved pointer/metadata tuple.
It expects `31 7` after actual source copying and cloning. No caller-location
intrinsic or panic handler is fabricated.
