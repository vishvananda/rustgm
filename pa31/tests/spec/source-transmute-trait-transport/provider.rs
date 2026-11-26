use crate::marker::{ConstParamTy_, StructuralPartialEq};

impl StructuralPartialEq for bool {}
impl cmp::Eq for bool {}
impl ConstParamTy_ for bool {}

#[unstable(feature = "transmutability", issue = "99571")]
#[lang = "transmute_opts"]
pub struct Assume {
    pub alignment: bool,
    pub lifetimes: bool,
    pub safety: bool,
    pub validity: bool,
}

impl PartialEq for Assume {
    fn eq(&self, other: &Self) -> bool {
        self.alignment == other.alignment
            && self.lifetimes == other.lifetimes
            && self.safety == other.safety
            && self.validity == other.validity
    }
}

impl cmp::Eq for Assume {}
impl StructuralPartialEq for Assume {}

#[unstable(feature = "transmutability", issue = "99571")]
#[unstable_feature_bound(transmutability)]
impl ConstParamTy_ for Assume {}

#[unstable(feature = "transmutability", issue = "99571")]
#[unstable_feature_bound(transmutability)]
#[lang = "transmute_trait"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
#[rustc_coinductive]
pub unsafe trait TransmuteFrom<Src: ?Sized, const ASSUME: Assume> {}
