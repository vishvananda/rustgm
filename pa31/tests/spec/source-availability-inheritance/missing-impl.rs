#[stable(feature = "reachable_policy", since = "1.0.0")]
pub struct Visible;

#[stable(feature = "reachable_policy", since = "1.0.0")]
pub trait Reachable {}

impl Reachable for Visible {}
