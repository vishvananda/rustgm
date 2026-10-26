pub mod atomic_ordering;
pub mod bindings;
pub mod call_traits;
pub mod comparisons;
pub mod constness;
pub mod crates;
pub(crate) mod decorations;
pub mod edition;
pub mod hash;
pub(crate) mod intern_slice;
pub mod intrinsics;
pub mod iteration;
pub mod language_hooks;
pub mod library_types;
pub mod linkage;
pub mod marker_traits;
pub mod names;
pub(crate) mod quoted;
pub(crate) mod radix_map;
pub(crate) mod retained;
pub mod runtime_checks;
pub mod source;

pub mod diagnostics;

pub(crate) mod attribute_catalog;
