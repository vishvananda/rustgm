//! The actual fresh named executable exposes imported results to a real C observer.
use super::*;

const CALLER: &str = r#"#![feature(no_core)]
#![no_core]
unsafe extern "C" { fn observe(value:u8); }
#[unsafe(no_mangle)]
pub extern "C" fn provided_observed()->u8 {
    let value=<u8 as base::Holder<u8>>::carry::<u8>(7u8);
    let borrowed=<u8 as base::Holder<u8>>::borrow(&value);
    *borrowed
}
fn main(){unsafe{observe(provided_observed());}}
"#;
const OBSERVER: &str = "#include <stdint.h>\n#include <stdio.h>\nvoid observe(uint8_t value){printf(\"%u\\n\",(unsigned)value);}\n";
const INDEPENDENT: &str = "#include <stdint.h>\nextern uint8_t provided_observed(void);\nextern void observe(uint8_t);\nint main(void){observe(provided_observed());return 0;}\n";

pub(super) fn actual_instances(
    report: &crate::resolve::expansion::Report,
    facts: &crate::sema::model::Declarations,
    program: &crate::lowir::program::Program,
) -> std::result::Result<(), crate::resolve::package::Error> {
    use crate::resolve::package::Error;
    let imported = report
        .resolution
        .crates
        .as_deref()
        .ok_or_else(|| Error::unlocated("actual imported instance inventory"))?;
    let linkage = facts
        .crates
        .as_deref()
        .ok_or_else(|| Error::unlocated("actual instance linkage"))?;
    let catalog = linkage
        .origin_catalog()
        .ok_or_else(|| Error::unlocated("actual instance Catalog"))?;
    let mut count = 0;
    for definition in imported.generic_definitions() {
        let definition = definition.map_err(Error::unlocated)?;
        let actual = &report.resolution.definitions[definition.0 as usize];
        let name = report.package.names.get(actual.name);
        if actual.kind != crate::resolve::model::DefKind::Function
            || !matches!(name, "carry" | "borrow")
        {
            continue;
        }
        let declaration = catalog.declaration(definition).map_err(Error::unlocated)?;
        let header = facts
            .function_header(definition)
            .map_err(Error::unlocated)?
            .ok_or_else(|| Error::unlocated("actual demanded original header"))?;
        assert_eq!(
            header.declaration,
            crate::resolve::body_origin::DeclarationOrigin::Imported(declaration)
        );
        assert!(header.body.is_some_and(|body| body.source().is_none()));
        assert!(header.environment.is_some());
        let symbol = linkage
            .rust_symbol(definition)
            .ok_or_else(|| Error::unlocated("actual trait member symbol"))?;
        let prefix = format!("{symbol}.i");
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|row| row.body.is_some() && row.name.starts_with(&prefix))
                .count(),
            1,
            "{name}"
        );
        eprintln!("actual imported {name} declaration={declaration:?} instance-prefix={prefix}");
        count += 1;
    }
    assert_eq!(count, 2);
    Ok(())
}

#[test]
fn named_provided_carry_and_borrow_prepare_observable_native7() -> Result {
    let mut work = ReservationWork::default();
    let (artifact, _) = artifact_with_alias(HOLDER, Some("base"), &mut work)?;
    let context = context(artifact.inventory())?;
    let config = Config::default();
    let mut sources = NoFiles;
    let request = ConsumerRequest::new(input(CALLER)?, &mut sources, &config, &context);
    let stage = ProofSession::from_collected(artifact, request, &mut work)?;
    let holder = named_trait(&stage, "Holder", &mut work)?;
    let (stage, status) = stage.trait_owner(holder, &mut work)?;
    assert_eq!(status, Demand::Ready(()));
    let stage = method_headers(stage, &mut work)?;
    let (_, program) = stage.execute(&mut work, |report, facts, entry, preparation| {
        let program = crate::lower::instances::build_selected_prepared(
            report,
            facts,
            entry,
            crate::pipeline::Prepared::default(),
            preparation,
        )?;
        actual_instances(report, facts, &program)?;
        actual_normalization_owner(report, facts, preparation)?;
        Ok(program)
    })?;
    assert_eq!(
        program
            .symbols
            .iter()
            .filter(|row| row.name == "provided_observed" && row.body.is_some())
            .count(),
        1
    );
    assert_eq!(
        program
            .symbols
            .iter()
            .filter(|row| row.name == "observe" && row.body.is_none())
            .count(),
        1
    );
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/selected-provided");
    std::fs::create_dir_all(&folder)?;
    let mut provider = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    provider.push_str(HOLDER);
    std::fs::write(folder.join("provider.rs"), provider)?;
    std::fs::write(folder.join("caller.rs"), CALLER)?;
    std::fs::write(folder.join("observer.c"), OBSERVER)?;
    std::fs::write(folder.join("independent-main.c"), INDEPENDENT)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("consumer.lowir"))?,
    )?;
    Ok(())
}
