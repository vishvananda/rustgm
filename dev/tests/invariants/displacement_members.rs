//! Original builtin members join source provenance to fresh reached layout calls.
use crate::{pipeline, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn input() -> Result<String> {
    Ok(test_source::fixture(
        "pa31/tests/spec/source-offset-of/provider/provider.rs",
    )?)
}
fn field_input(extra: &str) -> Result<String> {
    let source = input()?;
    let (header, _) = source
        .split_once("fn payload<B>")
        .ok_or("original offset provider boundary changed")?;
    Ok(format!("{header}\n{extra}\n"))
}
fn build(source: &str) -> Result<crate::lowir::program::Program> {
    let report = test_source::source_provider(source, "offset_members")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn original_offset_macro_keeps_open_parent_and_distinct_field_operands() -> Result {
    assert_eq!(size_of::<crate::sema::model::Operation>(), 16);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    let source = input()?;
    let program = build(&source)?;
    for name in [
        "option_u8",
        "option_u32",
        "option_u64",
        "option_ref",
        "field_first",
        "field_second",
        "field_third",
    ] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|symbol| symbol.name == name && symbol.body.is_some())
                .count(),
            1,
            "{name}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/offset-members-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn enum_offset_requires_crate_request_even_with_macro_allowance() -> Result {
    let source = input()?.replace(",offset_of_enum,", ",").replace(
        "allow_internal_unstable(builtin_syntax, core_intrinsics)",
        "allow_internal_unstable(builtin_syntax, core_intrinsics, offset_of_enum)",
    );
    let error = build(&source)
        .err()
        .ok_or("macro allowance supplied crate enum authority")?
        .to_string();
    assert!(
        error.contains("enum offset requires the crate's offset_of_enum"),
        "{error}"
    );
    Ok(())
}
#[test]
fn original_offset_field_privacy_and_no_autoderef_are_checked() -> Result {
    for (extra, expected) in [
        (
            "mod hidden {pub struct Secret {field:u8}}\n#[unsafe(no_mangle)]pub extern \"C\" fn private_offset()->usize{offset_of!(hidden::Secret,field)}",
            "aggregate field is private",
        ),
        (
            "#[unsafe(no_mangle)]pub extern \"C\" fn reference_offset()->usize{offset_of!(&Fields,second)}",
            "offset container must be nominal without autoderef",
        ),
        (
            "#[unsafe(no_mangle)]pub extern \"C\" fn nested_offset()->usize{offset_of!(Fields,first.0)}",
            "nested offset fields are not implemented",
        ),
    ] {
        let source = field_input(extra)?;
        let error = build(&source)
            .err()
            .ok_or("invalid offset source accepted")?
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
    Ok(())
}
#[test]
fn original_offset_intrinsic_uses_exact_macro_api_allowance() -> Result {
    let source = field_input(
        "#[unsafe(no_mangle)]pub extern \"C\" fn field_api()->usize{offset_of!(Fields,second)}",
    )?
    .replace(
        "allow_internal_unstable(builtin_syntax, core_intrinsics)",
        "allow_internal_unstable(builtin_syntax)",
    );
    let error = build(&source)
        .err()
        .ok_or("offset intrinsic accepted absent its actual API allowance")?
        .to_string();
    assert!(
        error.contains(
            "source intrinsic const API requires its checked feature or exact token allowance"
        ),
        "{error}"
    );
    Ok(())
}
#[test]
fn offset_member_names_keep_raw_and_unicode_normalization() -> Result {
    let source = field_input(
        "#[repr(C)]pub struct Accents{pub café:u64,pub r#type:u8}\n#[unsafe(no_mangle)]pub extern \"C\" fn unicode_offset()->usize{offset_of!(Accents,cafe\u{301})}\n#[unsafe(no_mangle)]pub extern \"C\" fn raw_offset()->usize{offset_of!(Accents,r#type)}",
    )?;
    let program = build(&source)?;
    for name in ["unicode_offset", "raw_offset"] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some())
        );
    }
    Ok(())
}
#[test]
fn offset_plan_handles_do_not_transfer_source_owner_or_member_authority() -> Result {
    let report = test_source::source_provider(&input()?, "offset_plan_ownership")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let rows = &facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.call_contexts.as_deref())
        .and_then(|contexts| contexts.offsets.as_deref())
        .ok_or("offset source rows missing")?
        .rows;
    assert!(rows.len() >= 4);
    let first = original_container(&facts, rows)?;
    let different = *rows
        .iter()
        .find(|plan| plan.field != first.field && plan.source_use != first.source_use)
        .ok_or("opposing actual offset owner missing")?;
    let Some(crate::sema::model::Operation::OffsetOf { plan: id }) =
        facts.node(first.node).and_then(|node| node.operation)
    else {
        return Err("offset plan handle missing".into());
    };
    let owner = first.source_use.owner(&facts)?;
    crate::sema::displacement_plans::checked(&report, &facts, owner, first.node, id)?;
    assert!(
        crate::sema::displacement_plans::checked(
            &report,
            &facts,
            different.source_use.owner(&facts)?,
            first.node,
            id
        )
        .is_err()
    );
    assert!(
        crate::sema::displacement_plans::checked(&report, &facts, owner, different.node, id)
            .is_err()
    );
    let rows = &mut facts
        .source_bindings
        .as_deref_mut()
        .and_then(|catalog| catalog.call_contexts.as_deref_mut())
        .and_then(|contexts| contexts.offsets.as_deref_mut())
        .ok_or("offset source rows missing")?
        .rows;
    rows[id.index() as usize].field = different.field;
    assert!(
        crate::sema::displacement_plans::checked(&report, &facts, owner, first.node, id).is_err()
    );
    reject_changed_coordinates(&report, &mut facts, first, id)?;
    reject_changed_annotation(&report, &mut facts, first, id)?;
    Ok(())
}
fn original_container(
    facts: &crate::sema::model::Declarations,
    rows: &[super::Plan],
) -> Result<super::Plan> {
    let (plan, parameter) = rows
        .iter()
        .find_map(|plan| {
            let crate::types::Kind::Adt(owner, arguments) = facts.types.kind(plan.container)?
            else {
                return None;
            };
            facts.types.arguments(arguments)?.find_map(|argument| {
                let crate::types::arguments::Argument::Type(ty) = argument else {
                    return None;
                };
                let crate::types::Kind::Parameter(id) = facts.types.kind(ty)? else {
                    return None;
                };
                let parameter = facts.types.parameter(id)?;
                (parameter.owner != owner).then_some((*plan, parameter))
            })
        })
        .ok_or("original open offset container missing")?;
    let owner = plan.source_use.owner(facts)?;
    let declaration = facts
        .inline_constants
        .as_deref()
        .and_then(|registry| registry.get(owner))
        .and_then(|body| body.declaration)
        .ok_or("original offset inline declaration missing")?;
    assert_eq!(parameter.owner.0, declaration.0);
    Ok(plan)
}
fn reject_changed_annotation(
    report: &crate::resolve::expansion::Report,
    facts: &mut crate::sema::model::Declarations,
    plan: super::Plan,
    id: super::OffsetPlanId,
) -> Result {
    let crate::syntax::ast::Kind::OffsetOf { container, .. } = report
        .package
        .unit(plan.node.file)
        .configured
        .ast
        .node(plan.node.node)
        .kind
    else {
        return Err("actual offset source missing".into());
    };
    let key = crate::resolve::unit::NodeKey {
        node: container,
        ..plan.node
    };
    let owner = plan.source_use.owner(facts)?;
    for ty in [
        None,
        Some(crate::types::TypeId::primitive(crate::types::Primitive::U8)),
    ] {
        facts
            .node_mut(key)
            .ok_or("actual offset annotation missing")?
            .ty = ty;
        assert!(
            crate::sema::displacement_plans::checked(report, facts, owner, plan.node, id).is_err()
        );
    }
    facts
        .node_mut(key)
        .ok_or("actual offset annotation missing")?
        .ty = Some(plan.container);
    Ok(())
}
fn reject_changed_coordinates(
    report: &crate::resolve::expansion::Report,
    facts: &mut crate::sema::model::Declarations,
    plan: super::Plan,
    id: super::OffsetPlanId,
) -> Result {
    facts
        .source_bindings
        .as_deref_mut()
        .and_then(|catalog| catalog.call_contexts.as_deref_mut())
        .and_then(|contexts| contexts.offsets.as_deref_mut())
        .ok_or("offset source rows missing")?
        .rows[id.index() as usize] = plan;
    let owner = plan.source_use.owner(facts)?;
    let variant = facts
        .members
        .variant(plan.field)
        .ok_or("actual offset variant missing")?;
    let original = facts.members.variants[variant.0 as usize];
    facts.members.variants[variant.0 as usize].ordinal = u32::MAX;
    assert!(crate::sema::displacement_plans::checked(report, facts, owner, plan.node, id).is_err());
    facts.members.variants[variant.0 as usize] = original;
    facts.members.variants[variant.0 as usize].end = original.start;
    assert!(crate::sema::displacement_plans::checked(report, facts, owner, plan.node, id).is_err());
    facts.members.variants[variant.0 as usize] = original;
    Ok(())
}
#[test]
fn offset_name_work_is_paid_on_cold_warm_and_rejected_widths() -> Result {
    let mut previous = 0;
    for width in [8_usize, 32, 128] {
        let mut names = crate::support::names::Names::default();
        let spelling = "x".repeat(width);
        let mut cold = 0_usize;
        let id = names.intern_paid(&spelling, &mut |growth| {
            cold += growth.work;
            Ok(())
        })?;
        let mut warm = 0_usize;
        assert_eq!(
            id,
            names.intern_paid(&spelling, &mut |growth| {
                warm += growth.work;
                assert_eq!(growth.bytes, 0);
                Ok(())
            })?
        );
        assert!(cold >= width && warm >= width && warm > previous);
        previous = warm;
        let mut rejected = 0_usize;
        assert!(
            names
                .intern_paid(&format!("{spelling}z"), &mut |growth| {
                    rejected += growth.work;
                    Err("name budget exhausted")
                })
                .is_err()
        );
        assert!(rejected > width);
        assert_eq!(names.len(), 1);
    }
    let mut names = crate::support::names::Names::default();
    let mut rejected = 0;
    assert!(
        names
            .intern_paid("cafe\u{301}", &mut |growth| {
                rejected += growth.work;
                if growth.bytes == 0 {
                    Ok(())
                } else {
                    Err("normalization budget exhausted")
                }
            })
            .is_err()
    );
    assert!(rejected > 5);
    assert!(names.is_empty());
    Ok(())
}
