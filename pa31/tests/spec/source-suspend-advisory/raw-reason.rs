#![feature(must_not_suspend)]
#[must_not_suspend=r#"holding a guard"#]pub struct S;
