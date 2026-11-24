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
impl<T: PointeeSized> Clone for &T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for &T {}

pub const trait Scale {
    fn factor(&self) -> u32;

    fn doubled(&self) -> u32 {
        match self.factor() {
            5 => 10,
            6 => 12,
            9 => 18,
            other => other,
        }
    }

    #[rustc_non_const_trait_method]
    fn runtime_bonus(&self) -> u32 {
        match self.factor() {
            5 => 105,
            9 => 109,
            other => other,
        }
    }
}

pub const trait Tagged: [const] Scale {
    fn tag(&self) -> u32;
}

pub const unsafe trait UnsafeScale {
    fn raw_factor(&self) -> u32;
}

pub struct Fixed(pub u32);

impl Clone for Fixed {
    fn clone(&self) -> Self {
        Fixed(self.0)
    }
}
impl Copy for Fixed {}

const impl Scale for Fixed {
    fn factor(&self) -> u32 {
        self.0
    }
}

const impl Tagged for Fixed {
    fn tag(&self) -> u32 {
        match self.doubled() {
            10 => 11,
            other => other,
        }
    }
}

const unsafe impl UnsafeScale for Fixed {
    fn raw_factor(&self) -> u32 {
        match self.0 {
            5 => 8,
            other => other,
        }
    }
}

pub struct Dynamic(pub u32);

impl Scale for Dynamic {
    fn factor(&self) -> u32 {
        self.0
    }
}

pub struct RefScale<'a>(pub &'a u32);

const impl<'a> Scale for RefScale<'a> {
    fn factor(&self) -> u32 {
        *self.0
    }
}

pub const fn scale_twice<T: Copy + [const] Scale>(value: T) -> u32 {
    value.doubled()
}

pub const fn apply_tag<T: Copy + [const] Tagged>(value: T) -> u32 {
    value.tag()
}

pub const fn strict_scale<T: Copy + const Scale>(value: T) -> u32 {
    value.factor()
}
