#[stable(feature = "alloc_layout", since = "1.28.0")]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[lang = "alloc_layout"]
pub struct Layout {
    // size of the requested block of memory, measured in bytes.
    size: usize,

    // alignment of the requested block of memory, measured in bytes.
    // we ensure that this is always a power-of-two, because API's
    // like `posix_memalign` require it and it is a reasonable
    // constraint to impose on Layout constructors.
    //
    // (However, we do not analogously require `align >= sizeof(void*)`,
    //  even though that is *also* a requirement of `posix_memalign`.)
    align: Alignment,
}

