//! External source instances inherit their declaration's lexical policy.
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    sema,
    support::source::SourceFile,
};
use std::path::Path;

const CASES: &[(&str, bool, &str, &str)] = &[
    (
        "external-inherit",
        false,
        "#![deny(unsafe_op_in_unsafe_fn)]mod m;",
        "pub unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "external-outer-override",
        true,
        "#![deny(unsafe_op_in_unsafe_fn)]#[allow(unsafe_op_in_unsafe_fn)]mod m;",
        "pub unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "external-inner-override",
        true,
        "#![deny(unsafe_op_in_unsafe_fn)]mod m;",
        "#![allow(unsafe_op_in_unsafe_fn)]pub unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "external-forbid",
        false,
        "#![forbid(unsafe_op_in_unsafe_fn)]mod m;",
        "#![allow(unsafe_op_in_unsafe_fn)]pub fn f(){}",
    ),
];
struct Modules<'a>(&'a str);
impl Sources for Modules<'_> {
    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
        if path == Path::new("m.rs") {
            Ok(Some(InputFile {
                path: path.into(),
                identity: path.into(),
                source: SourceFile::new(self.0.as_bytes().to_vec())?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[test]
fn module_policy_inherits_and_applies_inner_overrides() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, root, child) in CASES {
        let expanded = resolve::expansion::analyze(
            InputFile {
                path: "lib.rs".into(),
                identity: "lib.rs".into(),
                source: SourceFile::new(root.as_bytes().to_vec())?,
            },
            &mut Modules(child),
            &Config::default(),
        )?;
        let report = resolve::analyze_expanded(expanded)?;
        let result = sema::declarations::analyze(&report)
            .and_then(|facts| sema::bodies::analyze(&report, facts));
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {}",
            result.err().map_or_else(String::new, |e| e.to_string())
        );
    }
    Ok(())
}
