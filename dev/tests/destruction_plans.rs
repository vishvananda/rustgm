use rustgm::{
    body::{
        self,
        ir::{Body, LocalId, Owner, Place, Selection},
    },
    borrow::{
        destruction::{
            self, Condition, Operation, Plan,
            glue::{Catalog, Shape},
        },
        ownership::initialization,
    },
    resolve::expansion::Report,
};
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const D: &str = "struct D;impl Drop for D{fn drop(&mut self){}}";
fn local(body: &Body, report: &Report, name: &str) -> Result<LocalId, &'static str> {
    let index = body
        .locals
        .iter()
        .position(|local| {
            local.definition.is_some_and(|id| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[id.0 as usize].name)
                    == name
            })
        })
        .ok_or("missing local")?;
    Ok(LocalId(
        u32::try_from(index).map_err(|_| "too many locals")?,
    ))
}
fn inspect(
    source: &str,
    mut check: impl FnMut(&Body, &Report, &Catalog<'_>, &Plan) -> Result<(), &'static str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let mut catalog = Catalog::new(
        &facts.types,
        &facts.aggregates,
        &facts.members,
        &program.drops.as_ref().ok_or("pending methods")?.methods,
    )?;
    for cfg in &program.bodies {
        let init = initialization::analyze(
            &facts.types,
            &facts.aggregates,
            &facts.derives,
            &mut facts.copies,
            cfg,
        )
        .map_err(|e| e.message)?;
        let plan = destruction::select(&mut catalog, &init).map_err(|e| e.message)?;
        if matches!(cfg.owner, Owner::Function { definition, .. }
            if report.package.names.get(report.resolution.definitions[definition.0 as usize].name)=="f")
        {
            check(cfg, &report, &catalog, &plan)?;
        }
    }
    Ok(())
}
fn cleanup(plan: &Plan, local: LocalId) -> Result<&destruction::Cleanup, &'static str> {
    plan.cleanups
        .iter()
        .find(|c| {
            matches!(c.candidate.place,Place::Local(id) if id==local)
                && matches!(c.candidate.cause, initialization::Cause::Scope(_))
                && c.live_type.is_some()
        })
        .ok_or("missing live local cleanup")
}

#[test]
fn conditional_whole_objects_use_one_flag_and_partial_products_keep_field_order()
-> Result<(), Box<dyn std::error::Error>> {
    let source = format!("{D}fn f(flag:bool){{let mut x:D;if flag{{x=D;}}}}");
    rustc::qualify("conditional-drop-plan", &source, true)?;
    inspect(&source, |cfg, report, catalog, plan| {
        let c = cleanup(plan, local(cfg, report, "x")?)?;
        let node = &plan.nodes[c.node.ok_or("missing conditional node")?.0 as usize];
        assert!(matches!(node.condition, Condition::Initialized(_)));
        let Operation::Glue(id) = node.operation else {
            return Err("whole object not using shared glue");
        };
        assert!(catalog.get(id).ok_or("missing glue")?.method.is_some());
        assert_eq!(plan.flags.len(), 1);
        Ok(())
    })?;
    let source = format!("{D}fn f(flag:bool){{let pair=(D,D);if flag{{let _moved=pair.0;}}}}");
    rustc::qualify("partial-product-drop-plan", &source, true)?;
    inspect(&source, |cfg, report, _, plan| {
        let c = cleanup(plan, local(cfg, report, "pair")?)?;
        let node = &plan.nodes[c.node.ok_or("missing product node")?.0 as usize];
        let Operation::Fields(range) = node.operation else {
            return Err("partial product must split fields");
        };
        let fields = &plan.fields[range.start as usize..range.end as usize];
        assert_eq!(
            fields.iter().map(|f| f.selection).collect::<Vec<_>>(),
            [Selection::Tuple(0), Selection::Tuple(1)]
        );
        assert!(matches!(
            plan.nodes[fields[0].node.0 as usize].condition,
            Condition::Initialized(_)
        ));
        assert_eq!(
            plan.nodes[fields[1].node.0 as usize].condition,
            Condition::Always
        );
        assert_eq!(plan.flags.len(), 1);
        Ok(())
    })
}

#[test]
fn sparse_array_plans_keep_gap_loops_and_conditions_outside_them()
-> Result<(), Box<dyn std::error::Error>> {
    let mut sizes = None;
    for length in [8, 1_000_000_000] {
        let source = format!(
            "{D}fn f(flag:bool,items:[D;{length}]){{if flag{{let [first,..,last]=items;}}}}"
        );
        rustc::qualify(&format!("array-drop-plan-{length}"), &source, true)?;
        inspect(&source, |cfg, report, catalog, plan| {
            let c = cleanup(plan, local(cfg, report, "items")?)?;
            let root = &plan.nodes[c.node.ok_or("missing array plan")?.0 as usize];
            let Operation::Array(range) = root.operation else {
                return Err("missing sparse array plan");
            };
            let intervals = &plan.intervals[range.start as usize..range.end as usize];
            assert_eq!(
                intervals
                    .iter()
                    .map(|r| (r.start, r.end))
                    .collect::<Vec<_>>(),
                [(0, 1), (1, length - 1), (length - 1, length)]
            );
            assert!(matches!(intervals[0].condition, Condition::Initialized(_)));
            assert_eq!(intervals[1].condition, Condition::Always);
            assert!(matches!(intervals[2].condition, Condition::Initialized(_)));
            for interval in intervals {
                let element = &plan.nodes[interval.element.0 as usize];
                assert_eq!(element.condition, Condition::Always);
                assert!(matches!(element.operation, Operation::Glue(_)));
            }
            assert_eq!(plan.flags.len(), 2);
            let counts = (
                plan.nodes.len(),
                plan.fields.len(),
                plan.intervals.len(),
                catalog.recipes().len(),
                plan.steps,
            );
            if let Some(previous) = sizes {
                assert_eq!(counts, previous);
            } else {
                sizes = Some(counts);
            }
            Ok(())
        })?;
    }
    Ok(())
}

#[test]
fn selected_array_elements_preserve_partially_moved_subfields()
-> Result<(), Box<dyn std::error::Error>> {
    let source = format!("{D}fn f(items:[(D,D);1000000000]){{let [(first,_),..]=items;}}");
    rustc::qualify("nested-array-drop-plan", &source, true)?;
    inspect(&source, |cfg, report, _, plan| {
        let c = cleanup(plan, local(cfg, report, "items")?)?;
        let Operation::Array(range) = plan.nodes[c.node.ok_or("array")?.0 as usize].operation
        else {
            return Err("array");
        };
        let intervals = &plan.intervals[range.start as usize..range.end as usize];
        assert_eq!(intervals.len(), 2);
        let Operation::Fields(fields) = plan.nodes[intervals[0].element.0 as usize].operation
        else {
            return Err("partial element");
        };
        let children = &plan.fields[fields.start as usize..fields.end as usize];
        assert!(matches!(
            plan.nodes[children[0].node.0 as usize].operation,
            Operation::Empty
        ));
        assert!(matches!(
            plan.nodes[children[1].node.0 as usize].operation,
            Operation::Glue(_)
        ));
        assert_eq!((intervals[1].start, intervals[1].end), (1, 1_000_000_000));
        assert!(plan.flags.is_empty());
        Ok(())
    })
}

#[test]
fn enum_cleanup_dispatches_before_accessing_selected_variant_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let source = format!(
        "{D}enum E{{A(D,D),B(D)}}fn f(e:E){{match e{{E::A(first,_)=>{{}},E::B(_)=>{{}}}}}}"
    );
    rustc::qualify("enum-drop-plan", &source, true)?;
    inspect(&source, |cfg, report, catalog, plan| {
        let c = cleanup(plan, local(cfg, report, "e")?)?;
        let Operation::Variants(range) = plan.nodes[c.node.ok_or("enum")?.0 as usize].operation
        else {
            return Err("enum");
        };
        let variants = &plan.variants[range.start as usize..range.end as usize];
        assert_eq!(variants.len(), 2);
        for variant in variants {
            let Operation::Fields(fields) = plan.nodes[variant.node.0 as usize].operation else {
                return Err("enum field group");
            };
            assert!(fields.start < fields.end);
        }
        assert!(
            catalog
                .recipes()
                .iter()
                .any(|r| matches!(r.kind, Shape::Variants(_)))
        );
        Ok(())
    })
}

#[test]
fn drop_check_lifetimes_survive_field_moves_even_without_runtime_field_calls()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn check(_:bool){}";
    for (name, transfer, accepted) in [
        ("fields", "let (_a,_b)=pair;", false),
        ("whole", "let _all=pair;", true),
    ] {
        let source = format!(
            "{prefix}fn f(){{let mut a=0;let mut b=0;{{let pair=(D(&mut a),D(&mut b));{{{transfer}}}check(a==1&&b==1);}}}}"
        );
        rustc::qualify(&format!("plan-drop-liveness-{name}"), &source, accepted)?;
        inspect(&source, |cfg, report, _, plan| {
            let pair = local(cfg, report, "pair")?;
            let candidates: Vec<_> = plan
                .cleanups
                .iter()
                .filter(|c| {
                    matches!(c.candidate.place,Place::Local(id)if id==pair)
                        && matches!(c.candidate.cause, initialization::Cause::Scope(_))
                })
                .collect();
            assert_eq!(candidates.len(), 1);
            assert!(candidates[0].node.is_none());
            assert_eq!(candidates[0].live_type.is_some(), !accepted);
            Ok(())
        })?;
    }
    Ok(())
}

#[test]
fn custom_destructor_field_moves_are_rejected_before_planning()
-> Result<(), Box<dyn std::error::Error>> {
    let source =
        "struct D;struct W(D);impl Drop for W{fn drop(&mut self){}}fn f(w:W){let _moved=w.0;}";
    rustc::qualify("plan-custom-field-move", source, false)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let cfg = program
        .bodies
        .iter()
        .find(|b| {
            matches!(b.owner,Owner::Function{definition,..}
        if report.package.names.get(report.resolution.definitions[definition.0 as usize].name)=="f")
        })
        .ok_or("f")?;
    let init = initialization::analyze(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut facts.copies,
        cfg,
    )
    .map_err(|e| e.message)?;
    let mut catalog = Catalog::new(
        &facts.types,
        &facts.aggregates,
        &facts.members,
        &program.drops.as_ref().ok_or("pending")?.methods,
    )?;
    let error = destruction::select(&mut catalog, &init)
        .err()
        .ok_or("accepted destructor field move")?;
    assert_eq!(
        error.message,
        "cannot move a non-Copy field out of a type that implements Drop"
    );
    Ok(())
}
