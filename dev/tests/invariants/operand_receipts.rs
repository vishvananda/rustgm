//! Shared RHS value factories retain complete instances, caller lifetimes and body proof.
use super::*;
fn source(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa26/tests/spec/shared-value-operands/{name}.rs"
    ))?)
}
#[test]
fn shared_add_sub_factories_keep_distinct_lifetimes_and_exact_permits() -> Result {
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(
        "fn add(a:u8,b:&u8)->u8{<u8 as core::ops::Add<&u8>>::add(a,b)}fn sub(a:u8,b:&u8)->u8{<u8 as core::ops::Sub<&u8>>::sub(a,b)}fn main(){let b=5u8;let _=add(2,&b);let _=sub(9,&b);}",
    )?;
    assert_eq!(selected.len(), 2);
    let empty = crate::body::templates::Templates::source(Vec::new());
    let checked = originals(&mut facts, &empty)?;
    let mut built = Vec::new();
    for selected in selected {
        assert_eq!(selected.descriptor.scalars()?.rhs, Rhs::SharedReference);
        let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected)?;
        assert_eq!(input.universals, 1);
        assert_eq!(body.locals[0].ty, TypeId::primitive(Primitive::U8));
        assert!(
            matches!(facts.types.kind(body.locals[1].ty),Some(Kind::Reference{mutable:false,referent,region})if referent==TypeId::primitive(Primitive::U8)&&region==RegionId::bound(0,0)?)
        );
        assert_eq!(body.projections.len(), 1);
        assert!(matches!(
            body.projections[0].kind,
            crate::body::ir::ProjectionKind::Dereference { mutable: false, .. }
        ));
        let permit = crate::body::primitive_methods::validate(&mut facts, selected, &body, &input)?;
        assert!(prove(&mut facts, &checked, body.clone(), &input, None).is_err());
        prove(&mut facts, &checked, body.clone(), &input, Some(permit))?;
        let mut changed = input.clone();
        changed.universals = 0;
        assert!(
            crate::body::primitive_methods::validate(&mut facts, selected, &body, &changed)
                .is_err()
        );
        let owned = Selected {
            descriptor: Descriptor {
                operands: Operands::Scalars(Scalars {
                    rhs: Rhs::Owned,
                    ..selected.descriptor.scalars()?
                }),
                ..selected.descriptor
            },
            ..selected
        };
        assert!(owned.validate(&mut facts).is_err());
        let wrong = Selected {
            descriptor: Descriptor {
                operation: Operation::Arithmetic(Hook::Mul),
                ..selected.descriptor
            },
            ..selected
        };
        assert!(wrong.validate(&mut facts).is_err());
        built.push((body, input, permit));
    }
    assert!(
        prove(
            &mut facts,
            &checked,
            built[0].0.clone(),
            &built[0].1,
            Some(built[1].2)
        )
        .is_err()
    );
    Ok(())
}
#[test]
fn static_reference_calls_preserve_zero_universals_and_canonical_signatures() -> Result {
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(&source("reified-static")?)?;
    assert_eq!(selected.len(), 2);
    for selected in selected {
        let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected)?;
        assert_eq!(input.universals, 0);
        assert!(matches!(
            facts.types.kind(body.locals[1].ty),
            Some(Kind::Reference {
                mutable: false,
                region: RegionId::STATIC,
                ..
            })
        ));
        crate::body::primitive_methods::validate(&mut facts, selected, &body, &input)?;
        let Kind::FunctionItem {
            item,
            signature,
            arguments,
        } = facts
            .types
            .kind(selected.instance.ty())
            .ok_or("actual item")?
        else {
            return Err("item".into());
        };
        let Kind::Function {
            parameters,
            result,
            convention,
            ..
        } = facts.types.kind(signature).ok_or("signature")?
        else {
            return Err("function".into());
        };
        for (unsafe_, bound_regions) in [(true, 0), (false, 1)] {
            let signature = facts.types.intern(Kind::Function {
                parameters,
                result,
                convention,
                unsafe_,
                bound_regions,
            })?;
            let ty = facts.types.intern(Kind::FunctionItem {
                item,
                signature,
                arguments,
            })?;
            assert!(
                Selected {
                    instance: Instance::new(&facts.types, ty)?,
                    ..selected
                }
                .validate(&mut facts)
                .is_err()
            );
        }
    }
    println!(
        "shared-value-layout Descriptor={} Selected={} Permit={} Body={} Operation={} Traits={} Resolution={}",
        size_of::<Descriptor>(),
        size_of::<Selected>(),
        size_of::<crate::body::primitive_methods::Permit>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::sema::model::Operation>(),
        size_of::<crate::resolve::model::BuiltinTraits>(),
        size_of::<crate::resolve::model::Resolution>()
    );
    assert_eq!(size_of::<Descriptor>(), 5);
    assert_eq!(size_of::<Selected>(), 12);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn repeated_static_shared_instances_reuse_two_paid_factories() -> Result {
    for width in [1, 8, 32] {
        let mut text = String::new();
        for n in 0..width {
            write!(
                text,
                "fn a{n}()->u8{{<u8 as core::ops::Add<&'static u8>>::add(2,&5)}}fn s{n}()->u8{{<u8 as core::ops::Sub<&'static u8>>::sub(9,&5)}}"
            )?;
        }
        text.push_str("fn main(){");
        for n in 0..width {
            write!(text, "let _=a{n}();let _=s{n}();")?;
        }
        text.push('}');
        let factory = fixture_for(&text)?;
        assert_eq!(factory.selected.len(), 2);
        assert!(factory.instructions < 500 + 50 * width);
        println!(
            "shared-value width={width} factories={} definitions={} instructions={} dispatch-selections={} inference={} registry-capacity={}",
            factory.selected.len(),
            factory.definitions,
            factory.instructions,
            factory.facts.dispatch_dependencies.work.selections,
            factory.facts.body_work.inference_steps,
            factory
                .facts
                .primitive_methods
                .as_ref()
                .ok_or("registry")?
                .selected
                .capacity()
        );
    }
    let report = crate::test_source::resolve("fn main(){let _=2u8+5u8;}", true)?;
    assert!(
        crate::pipeline::analyze_source(&report)?
            .facts
            .primitive_methods
            .is_none()
    );
    Ok(())
}
#[test]
fn source_owned_methods_execute_without_bootstrap_factory_evidence() -> Result {
    for name in ["source-owned", "source-absent"] {
        let text = crate::test_source::fixture(&format!(
            "pa31/tests/spec/shared-value-operands/{name}.rs"
        ))?;
        let error = crate::test_source::source_provider(&text, "actual_shared_source_lang")
            .and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            })
            .err()
            .ok_or("unsupported actual source role acquired bootstrap permission")?;
        assert!(
            error.contains("unsupported source language item"),
            "{error}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa26/shared-primitive-operands-v1/source-runtime");
    for (name, accepted) in [
        ("source-ordinary-absent", false),
        ("source-ordinary-owned", true),
    ] {
        let text = crate::test_source::fixture(&format!(
            "pa31/tests/spec/shared-value-operands/{name}.rs"
        ))?;
        let report = crate::test_source::source_provider(&text, "shared_source_owner")?;
        let analysis = crate::pipeline::analyze_source(&report);
        if !accepted {
            assert!(analysis.is_err());
            continue;
        }
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = analysis?;
        assert!(facts.primitive_methods.is_none());
        let roots = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .filter_map(|(i, d)| {
                (d.kind == crate::resolve::model::DefKind::Function
                    && matches!(report.package.names.get(d.name), "add_value" | "sub_value"))
                .then_some(u32::try_from(i).map(crate::resolve::model::DefId))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        assert_eq!(roots.len(), 2);
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &roots,
            },
            prepared,
        )?;
        assert!(facts.primitive_methods.is_none());
        let mut bytes = Vec::new();
        crate::lowir::writer::write(&program, &mut bytes)?;
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("program.lowir"), bytes)?;
        std::fs::write(folder.join("source.rs"), text)?;
    }
    Ok(())
}

#[test]
fn shared_numeric_rhs_narrows_only_existing_numeric_domains() -> Result {
    let Factory {
        mut facts,
        selected,
        ..
    } = fixture_for(&source("qualified")?)?;
    assert_eq!(selected.len(), 2);
    for selected in selected {
        let definition = selected.validate(&mut facts)?.definition;
        let owner = facts
            .associated
            .trait_owner(definition)
            .ok_or("actual owner")?;
        for domain in [
            infer::Domain::Integer,
            infer::Domain::Float,
            infer::Domain::Any,
        ] {
            for mutable in [false, true] {
                let mut context = Context::default();
                let left = context.known(&facts.types, TypeId::primitive(Primitive::U8))?;
                let element = context.variable(domain)?;
                let right = context.construct(
                    Constructor::Reference {
                        mutable,
                        region: RegionId::STATIC,
                    },
                    &[element],
                )?;
                let goal = Predicate::trait_application(
                    left,
                    crate::types::ItemId(owner.0),
                    vec![crate::types::infer::arguments::Argument::Type(right)],
                );
                let result = candidate(&facts, &mut context, &goal, false)?;
                if mutable {
                    assert!(matches!(result, Candidate::Unrelated));
                    assert!(context.head(&facts.types, element)?.is_none());
                } else if domain == infer::Domain::Integer {
                    assert!(matches!(result, Candidate::Present(_)));
                    assert!(matches!(
                        context.head(&facts.types, element)?,
                        Some(Constructor::Primitive(Primitive::U8))
                    ));
                } else {
                    assert!(matches!(result, Candidate::Deferred));
                    assert!(context.head(&facts.types, element)?.is_none());
                }
            }
        }
    }
    Ok(())
}

#[test]
fn generic_rhs_regions_keep_canonical_scoped_substitution() -> Result {
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(&source("generic")?)?;
    assert_eq!(selected.len(), 2);
    for selected in selected {
        let source = selected.validate(&mut facts)?;
        let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected)?;
        crate::body::primitive_methods::validate(&mut facts, selected, &body, &input)?;
        let Kind::FunctionItem {
            item, arguments, ..
        } = facts.types.kind(selected.instance.ty()).ok_or("item")?
        else {
            return Err("item".into());
        };
        let Kind::Function {
            parameters,
            result,
            convention,
            unsafe_,
            bound_regions,
        } = facts.types.kind(source.signature).ok_or("function")?
        else {
            return Err("function".into());
        };
        let parameters = facts.types.list(parameters).ok_or("parameters")?.to_vec();
        let wrong = facts.types.intern(Kind::Reference {
            referent: TypeId::primitive(Primitive::U8),
            mutable: false,
            region: RegionId::STATIC,
        })?;
        if parameters[1] != wrong {
            let parameters = facts.types.intern_list(&[parameters[0], wrong])?;
            let signature = facts.types.intern(Kind::Function {
                parameters,
                result,
                convention,
                unsafe_,
                bound_regions,
            })?;
            let ty = facts.types.intern(Kind::FunctionItem {
                item,
                arguments,
                signature,
            })?;
            assert!(
                Selected {
                    instance: Instance::new(&facts.types, ty)?,
                    ..selected
                }
                .validate(&mut facts)
                .is_err()
            );
        }
    }
    Ok(())
}
