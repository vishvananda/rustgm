//! Original source bodies reach the genuine user handler through checked imports.
use crate::{
    resolve::{
        self,
        load::{InputFile, Sources},
        model::{DefId, DefKind},
    },
    test_source,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const FIXTURE: &str = "pa31/tests/spec/source-panic-handler/";
const USER_PRELUDE: &str = "use crate::Option::Some;";
struct Inputs;
impl Sources for Inputs {
    fn read(&mut self, path: &std::path::Path) -> std::result::Result<Option<InputFile>, String> {
        if !path.ends_with("handler-user.rs") {
            return Ok(None);
        }
        let source = test_source::fixture(&(FIXTURE.to_owned() + "handler-user.rs"))
            .map_err(|e| e.to_string())?;
        let source = USER_PRELUDE.to_owned() + &source;
        Ok(Some(InputFile {
            identity: "handler-user.rs".into(),
            path: path.into(),
            source: crate::support::source::SourceFile::new(source.into_bytes())?,
        }))
    }
}
fn report() -> Result<resolve::expansion::Report> {
    let source = test_source::fixture(&(FIXTURE.to_owned() + "provider.rs"))?;
    source_report(&source)
}
fn source_report(source: &str) -> Result<resolve::expansion::Report> {
    Ok(test_source::source_provider_with_sources(
        source,
        "original_panic_handler",
        &crate::expand::config::Config::default(),
        &mut Inputs,
    )?)
}
#[test]
fn original_source_handler_location_and_abort_reach_checked_runtime() -> Result {
    let report = report()?;
    // This is explicit local execution. Unreached full Formatter bodies remain Pending.
    let names = [
        "run",
        "condition",
        "handle",
        "new",
        "line",
        "column",
        "location",
        "can_unwind",
        "force_no_backtrace",
        "caller",
        "panic_fmt",
        "panic",
        "from_str",
        "len",
        "as_bytes",
        "as_ptr",
        "metadata",
        "drop",
        "shl",
        "bitor",
        "not",
    ];
    let mut roots = Vec::new();
    let mut entry = None;
    for (index, row) in report.resolution.definitions.iter().enumerate() {
        let name = report.package.names.get(row.name);
        if row.kind != DefKind::Function || !names.contains(&name) {
            continue;
        }
        let Some(key) = row.node else {
            continue;
        };
        let ast = &report.package.unit(key.file).configured.ast;
        let crate::syntax::ast::Kind::Item(item) = ast.node(key.node).kind else {
            continue;
        };
        if !matches!(
            ast.items[item as usize].kind,
            crate::syntax::item::ItemKind::Function { body: Some(_), .. }
        ) {
            continue;
        }
        let id = DefId(u32::try_from(index)?);
        roots.push(id);
        if name == "run" {
            entry = Some(id);
        }
    }
    let entry = entry.ok_or("missing actual entry")?;
    assert!(roots.contains(&entry));
    let (mut facts, prepared) = crate::pipeline::prepare_local_runtime(&report, &roots)?;
    reject_foreign_handler_body(&report, &mut facts)?;
    reject_static_handler_signature(&report, &mut facts)?;
    let program =
        crate::lower::instances::build_local_prepared(&report, &mut facts, None, prepared)?;
    let handler = program
        .symbols
        .iter()
        .find(|symbol| symbol.name == "rust_begin_unwind")
        .ok_or("missing actual handler symbol")?;
    assert!(
        handler.body.is_some(),
        "source handler body was not lowered from its checked proof"
    );
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/source-panic-handler-v3/provider.lowir");
    std::fs::create_dir_all(output.parent().ok_or("output parent")?)?;
    crate::lowir::writer::write(&program, &mut std::fs::File::create(output)?)?;
    Ok(())
}

#[test]
fn source_weak_imports_remain_bodyless_and_reject_custom_linkage() -> Result {
    let source = test_source::fixture(&(FIXTURE.to_owned() + "provider.rs"))?;
    let object = source.replace("#[path=\"handler-user.rs\"]pub mod user;", "");
    let report = source_report(&object)?;
    let (facts, _) =
        crate::pipeline::prepare_local_runtime(&report, &[source_function(&report, "panic_fmt")?])?;
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == "panic_impl")
        .ok_or("actual weak source declaration")?;
    let id = DefId(u32::try_from(index)?);
    let registry = facts.foreign.as_deref().ok_or("actual foreign registry")?;
    let weak = registry.function(id).ok_or("actual weak callable")?;
    assert_eq!(weak.linkage, crate::support::linkage::Linkage::Import);
    let crate::support::linkage::Naming::Exact(name) = weak.naming else {
        return Err("weak source import lost its exact internal symbol".into());
    };
    assert_eq!(registry.names().get(name), Some("rust_begin_unwind"));
    assert!(!facts.typed_function_bodies.contains(&id));
    for name in ["panic_impl", "foreign_panic"] {
        let custom = object.replace(
            "#[lang = \"panic_impl\"]",
            &format!("#[lang = \"panic_impl\"]#[link_name=\"{name}\"]"),
        );
        let report = source_report(&custom)?;
        let error = crate::pipeline::prepare_local_runtime(
            &report,
            &[source_function(&report, "panic_fmt")?],
        )
        .err()
        .ok_or("custom weak linkage silently remapped")?;
        assert!(
            error.to_string().contains("custom weak panic linkage"),
            "{error}"
        );
    }
    let report = source_report(&source)?;
    let error =
        crate::pipeline::prepare_local_runtime(&report, &[source_function(&report, "panic_fmt")?])
            .err()
            .ok_or("handler body without demanded original gained permission")?;
    assert!(
        error.to_string().contains("completed actual local body"),
        "{error}"
    );
    Ok(())
}

fn source_function(report: &resolve::expansion::Report, name: &str) -> Result<DefId> {
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            row.kind == DefKind::Function && report.package.names.get(row.name) == name
        })
        .ok_or("actual source function")?;
    Ok(DefId(u32::try_from(index)?))
}
fn reject_foreign_handler_body(
    report: &resolve::expansion::Report,
    facts: &mut crate::sema::model::Declarations,
) -> Result {
    let id = source_function(report, "handle")?;
    let index = *facts
        .function_indices
        .get(&id)
        .ok_or("actual handler index")?;
    let original = facts.functions[index];
    let mut body = original.body.ok_or("actual handler body")?;
    let other = (0..report.package.units.len())
        .find(|&index| usize::try_from(body.file.0).ok() != Some(index))
        .ok_or("actual opposing source unit")?;
    body.file = crate::resolve::unit::FileId(u32::try_from(other)?);
    let before = facts
        .foreign
        .as_deref()
        .and_then(|registry| registry.function(id));
    facts.functions[index].body = Some(body);
    let error = super::complete(report, facts).err();
    facts.functions[index] = original;
    let error = error.ok_or("foreign file body gained handler linkage")?;
    assert!(
        error.to_string().contains("defining function qualifiers"),
        "{error}"
    );
    assert_eq!(
        before,
        facts
            .foreign
            .as_deref()
            .and_then(|registry| registry.function(id))
    );
    Ok(())
}
fn reject_static_handler_signature(
    report: &resolve::expansion::Report,
    facts: &mut crate::sema::model::Declarations,
) -> Result {
    use crate::types::{Kind, RegionId};
    let id = source_function(report, "handle")?;
    let mut function = facts.functions[*facts.function_indices.get(&id).ok_or("actual handler")?];
    let mut signature = facts
        .types
        .kind(function.signature)
        .ok_or("actual signature")?;
    let Kind::Function { parameters, .. } = &mut signature else {
        return Err("actual handler function type".into());
    };
    let parameter = *facts
        .types
        .list(*parameters)
        .and_then(|list| list.first())
        .ok_or("actual input")?;
    let Kind::Reference {
        mutable, referent, ..
    } = facts.types.kind(parameter).ok_or("actual reference")?
    else {
        return Err("actual handler reference type".into());
    };
    let parameter = facts.types.intern(Kind::Reference {
        mutable,
        referent,
        region: RegionId::STATIC,
    })?;
    *parameters = facts.types.intern_list(&[parameter])?;
    function.signature = facts.types.intern(signature)?;
    let info = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(super::LangItem::PanicInfo))
        .ok_or("actual PanicInfo")?;
    assert!(super::signature(facts, function, info, false).is_err());
    Ok(())
}
