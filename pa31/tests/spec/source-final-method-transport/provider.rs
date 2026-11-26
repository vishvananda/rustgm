#[cfg(rustc_oracle)]
#[lang = "freeze"]
pub unsafe auto trait Freeze {}

pub trait Score {
    final fn fixed(&self) -> u8 {
        41
    }

    fn open(&self) -> u8 {
        29
    }
}

impl Score for u8 {}
