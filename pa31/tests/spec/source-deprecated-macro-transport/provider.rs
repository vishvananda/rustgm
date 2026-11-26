#[macro_export]
#[stable(feature = "deprecation_fixture", since = "1.0.0")]
#[deprecated(since = "1.1.0", note = "prefer the replacement macro")]
macro_rules! old { () => { 7 }; }

#[allow(deprecated)]
#[stable(feature = "deprecation_fixture", since = "1.0.0")]
pub use crate::old as old_alias;

#[macro_export]
#[stable(feature = "deprecation_fixture", since = "1.0.0")]
#[deprecated(since = "TBD", note = "scheduled macro replacement")]
macro_rules! later { () => { 8 }; }

#[stable(feature = "deprecation_fixture", since = "1.0.0")]
pub fn modern() -> u8 { 9 }

#[stable(feature = "deprecation_fixture", since = "1.0.0")]
#[deprecated(since = "1.1.0", note = "binding-only annotation")]
pub use self::modern as modern_alias;

#[stable(feature = "deprecation_fixture", since = "1.0.0")]
#[deprecated(since = "1.1.0", note = "prefer the target replacement")]
pub fn legacy() -> u8 { 10 }

#[allow(deprecated)]
#[stable(feature = "deprecation_fixture", since = "1.0.0")]
#[deprecated(since = "1.1.0", note = "binding-only annotation")]
pub use self::legacy as legacy_alias;
