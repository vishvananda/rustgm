//! Original anonymous constants defer associated values until their exact instance.
use crate::provider_traits::lower;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[path = "anon_borrows/implied.rs"]
mod implied;
const PACKET: &str = "pa31/tests/spec/anonymous-associated-borrows";
const ORDINARY: &str = "pa25/tests/spec/anonymous-associated-borrows";

#[test]
fn inferred_local_array_outputs_reprove_private_debts_under_scalar_conditions() -> Result {
    outcomes("pa25/tests/spec/anonymous-array-outputs", 8, bootstrap)?;
    outcomes("pa31/tests/spec/anonymous-array-outputs", 8, lower)
}

#[test]
fn inferred_output_work_tracks_real_parent_ports() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let input = crate::resolve::fixture(&format!(
            "pa25/tests/spec/anonymous-array-outputs/scaling-{width}.rs"
        ))?;
        let report = crate::resolve::resolve(&input, true)?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        let work = [
            analysis.facts.body_work.inference_steps,
            analysis.facts.body_work.canonical_steps,
            analysis
                .facts
                .nested_outputs
                .as_ref()
                .map_or(0, |r| r.ports),
        ];
        assert_eq!(work[2], width);
        println!("inferred output width={width} work={work:?}");
        if let Some(old) = previous {
            for (before, after) in std::iter::zip(old, work) {
                assert!(after <= 5 * before + 512, "{old:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
fn source_inline_borrows_keep_deferred_associated_values() -> Result {
    outcomes(PACKET, 16, lower)
}

fn source_roles() -> Result<String> {
    let original = crate::resolve::fixture("pa31/tests/spec/cstr-source/local-const.rs")?;
    let (roles, _) = original.split_once("pub mod ffi").ok_or("source roles")?;
    Ok(format!("#![feature(const_destruct)]\n{roles}"))
}

#[test]
fn inline_raw_pointer_borrows_keep_child_region_identities() -> Result {
    let roles = source_roles()?;
    let source = format!(
        r"{roles}
        pub trait Project {{ type Target; }}
        const fn never<T>() -> *const T {{ loop {{}} }}
        pub fn inline<T>() -> *const T {{
            let borrowed: &T = const {{ unsafe {{ &*never::<T>() }} }};
            borrowed as *const T
        }}
        pub fn projected<D: Project>() {{
            let borrowed: &D::Target = const {{ unsafe {{ &*never::<D::Target>() }} }};
            let _ = borrowed;
        }}
        pub fn ordinary<T>() -> *const T {{
            let borrowed: &T = unsafe {{ &*never::<T>() }};
            borrowed as *const T
        }}
        fn main() {{}}
        "
    );
    lower(&source)?;
    let rejected = format!(
        "{source} pub fn escape<T>() -> &'static u8 {{ const {{ let local = 7u8; &local }} }}"
    );
    let error = lower(&rejected)
        .err()
        .ok_or("borrow of inline local escaped")?
        .to_string();
    assert!(error.contains("does not outlive"), "{error}");
    Ok(())
}

#[test]
fn inline_function_pointer_selection_waits_for_its_enclosing_instance() -> Result {
    let roles = source_roles()?;
    let source = format!(
        r#"{roles}
        pub trait Nested<T, I> {{ fn call(arg: I) -> Self; }}
        pub trait Source {{ const FLAG: bool; type Item; }}
        pub struct Container<T>(*const T);
        impl<T, I: Source<Item = T>> Nested<T, I> for Container<T> {{
            fn call(_: I) -> Self {{ Container(1usize as *const T) }}
        }}
        fn direct<T, I: Source<Item = T>>(_: I) -> Container<T> {{
            Container(2usize as *const T)
        }}
        pub fn collect<T, I: Source<Item = T>>(iterator: I) -> Container<T> {{
            let fun: fn(I) -> Container<T> = const {{
                if I::FLAG {{ direct }} else {{ Nested::<T, I>::call }}
            }};
            fun(iterator)
        }}
        struct Direct;
        impl Source for Direct {{ const FLAG: bool = true; type Item = u8; }}
        struct Fallback;
        impl Source for Fallback {{ const FLAG: bool = false; type Item = u8; }}
        #[unsafe(no_mangle)]
        pub extern "C" fn probe() {{
            let _ = collect::<u8, Direct>(Direct);
            let _ = collect::<u8, Fallback>(Fallback);
        }}
        "#
    );
    let program = lower(&source)?;
    assert!(program.symbols.iter().any(|symbol| symbol.body.is_some()));
    Ok(())
}

#[test]
fn bootstrap_inline_borrows_keep_ordinary_value_and_const_rules() -> Result {
    outcomes(ORDINARY, 5, bootstrap)
}

fn bootstrap(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::resolve(source, true)?;
    crate::provider_traits::lower_report(&report)
}

fn outcomes(
    packet: &str,
    expected_count: usize,
    prepare: impl Fn(&str) -> Result<rustgm::lowir::program::Program>,
) -> Result {
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{packet}/cases.tsv"))?.lines() {
        let columns: Vec<_> = line.split('\t').collect();
        let [path, expected, reason] = columns.as_slice() else {
            return Err("invalid anonymous borrow case".into());
        };
        let result = prepare(&crate::resolve::fixture(path)?);
        if *expected == "0" {
            if let Err(error) = result {
                failures.push(format!("{path}: {error}"));
            }
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("{path}: anonymous borrow opponent accepted"))?;
            assert!(error.to_string().contains(reason), "{path}: {error}");
        }
        count += 1;
    }
    assert_eq!(count, expected_count);
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn bootstrap_inline_borrows_execute_selected_values() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root
        .join("obj/parallel/initializer-completion/ordinary-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let observer = crate::resolve::fixture(&format!("{ORDINARY}/observer.c"))?;
    for name in ["generic-inline", "const-call", "concrete-inline"] {
        let source = crate::resolve::fixture(&format!("{ORDINARY}/{name}.rs"))?;
        crate::ffi_program::check(root, &folder, name, &bootstrap(&source)?, &observer, "7\n")?;
    }
    Ok(())
}

#[test]
fn projected_inline_original_retains_its_complete_source_contract() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/projected-inline.rs"))?;
    let report = crate::provider_traits::report(&source)?;
    rustgm::pipeline::analyze_source(&report)?;
    Ok(())
}

#[test]
fn anonymous_promotion_work_tracks_distinct_owners() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/borrowed-inline-size.rs"))?;
    let (prefix, _) = source
        .rsplit_once("pub fn generic<T>")
        .ok_or("anonymous borrow prefix")?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut text = prefix.to_owned();
        for index in 0..width {
            writeln!(
                text,
                "pub fn probe{index}<T>()->usize{{const{{let p=&<T as SizedTypeProperties>::SIZE;*p}}}}"
            )?;
        }
        let report = crate::provider_traits::report(&text)?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        let work = [
            analysis.facts.body_work.inference_steps,
            analysis.facts.body_work.canonical_steps,
            analysis.facts.constant_work.anonymous_templates,
        ];
        println!("anonymous borrows width={width} work={work:?}");
        if let Some(old) = previous {
            for (before, after) in std::iter::zip(old, work) {
                assert!(
                    after <= 5 * before + 512,
                    "width={width}: {old:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn anonymous_borrow_instances_execute_actual_associated_initializers() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root
        .join("obj/parallel/initializer-completion/native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        root,
        &folder,
        "anonymous",
        &lower(&crate::resolve::fixture(&format!("{PACKET}/native.rs"))?)?,
        &crate::resolve::fixture(&format!("{PACKET}/observer.c"))?,
        "4 8 4 4 8\n",
    )
}
