pub enum Outcome<T> {
    Value(T),
    Error(u8),
}
pub enum Short {
    Error(u8),
}
impl<T> Residual<T> for Short {
    type TryType = Outcome<T>;
}
impl<T> FromResidual<Short> for Outcome<T> {
    fn from_residual(value: Short) -> Self {
        match value {
            Short::Error(e) => Outcome::Error(e),
        }
    }
}
impl<T> Try for Outcome<T> {
    type Output = T;
    type Residual = Short;
    fn from_output(value: T) -> Self {
        Outcome::Value(value)
    }
    fn branch(self) -> ControlFlow<Short, T> {
        match self {
            Outcome::Value(v) => ControlFlow::Continue(v),
            Outcome::Error(e) => ControlFlow::Break(Short::Error(e)),
        }
    }
}

pub fn forward<'a>(value: Outcome<&'a u8>) -> Outcome<&'a u8> {
    let result = value?;
    Outcome::Value(result)
}

pub fn shorten<'a, 'b: 'a>(value: Outcome<&'b u8>) -> Outcome<&'a u8> {
    let result = value?;
    Outcome::Value(result)
}
