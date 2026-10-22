//! Declaration eligibility is shared across uses and never substitutes whole fields.
use crate::semantic;
use rustgm::types::{
    ItemId, Kind, Primitive, Store, TypeId,
    aggregate::{self, Definitions, Repr, Variant},
    arguments::Argument,
    parameters::{Parameter, ParameterKind},
    unsizing::Rules,
};
use std::fmt::Write as _;

fn parameter(types: &mut Store, owner: ItemId, index: u32) -> Result<TypeId, &'static str> {
    let id = types.intern_parameter(Parameter {
        owner,
        index,
        kind: ParameterKind::Type { sized: false },
    })?;
    types.intern(Kind::Parameter(id))
}
fn define(
    types: &mut Store,
    definitions: &mut Definitions,
    item: ItemId,
    fields: &[TypeId],
) -> Result<(), &'static str> {
    let fields = types.intern_list(fields)?;
    definitions.insert(
        item,
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )
}

#[test]
fn only_a_structural_tail_parameter_can_change() -> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut rules = Rules::default();
    let byte = TypeId::primitive(Primitive::U8);
    for item in 0..5 {
        let owner = ItemId(item);
        let tail = parameter(&mut types, owner, 2)?;
        let pointer = types.intern(Kind::Pointer {
            mutable: false,
            pointee: tail,
        })?;
        let tuple = types.intern_list(&[byte, tail])?;
        let tuple = types.intern(Kind::Tuple(tuple))?;
        let fields = match item {
            0 => vec![byte, tail],
            1 => vec![pointer, tail],
            2 => vec![byte, pointer],
            3 => vec![byte, tuple],
            _ => vec![byte, TypeId::primitive(Primitive::Str)],
        };
        define(&mut types, &mut definitions, owner, &fields)?;
        assert_eq!(
            rules.parameter(&types, &definitions, owner)?,
            (item == 0).then_some(2)
        );
    }
    assert_eq!(definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn nested_tail_rules_preserve_other_arguments() -> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut rules = Rules::default();
    let prefix = parameter(&mut types, ItemId(0), 0)?;
    let tail = parameter(&mut types, ItemId(0), 1)?;
    let prefix = types.intern(Kind::Pointer {
        mutable: false,
        pointee: prefix,
    })?;
    define(&mut types, &mut definitions, ItemId(0), &[prefix, tail])?;
    for item in 1..3 {
        let parameter = parameter(&mut types, ItemId(item), 0)?;
        let prefix = if item == 1 {
            TypeId::primitive(Primitive::U8)
        } else {
            parameter
        };
        let arguments =
            types.intern_arguments(&[Argument::Type(prefix), Argument::Type(parameter)])?;
        let tail = types.intern(Kind::Adt(ItemId(0), arguments))?;
        define(&mut types, &mut definitions, ItemId(item), &[tail])?;
        assert_eq!(
            rules.parameter(&types, &definitions, ItemId(item))?,
            (item == 1).then_some(0)
        );
    }
    Ok(())
}

#[test]
fn deep_shared_declarations_have_linear_work_and_cached_answers() -> Result<(), &'static str> {
    for count in [256, 512, 1024, 4096] {
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let mut rules = Rules::default();
        for item in 0..count {
            let parameter = parameter(&mut types, ItemId(item), 0)?;
            let tail = if item + 1 == count {
                parameter
            } else {
                let arguments = types.intern_arguments(&[Argument::Type(parameter)])?;
                types.intern(Kind::Adt(ItemId(item + 1), arguments))?
            };
            define(&mut types, &mut definitions, ItemId(item), &[tail])?;
        }
        assert_eq!(rules.parameter(&types, &definitions, ItemId(0))?, Some(0));
        assert!(rules.steps < count as usize * 15);
        let before = rules.steps;
        for item in 0..count {
            assert_eq!(
                rules.parameter(&types, &definitions, ItemId(item))?,
                Some(0)
            );
        }
        assert_eq!(rules.steps - before, count as usize);
        assert_eq!(definitions.instantiation_steps(), 0);
    }
    Ok(())
}

#[test]
fn projected_candidates_keep_mixed_slots_and_exclude_prefix_parameters()
-> Result<(), Box<dyn std::error::Error>> {
    for (prefix, expected) in [
        ("&'a u8", vec![1, 2]),
        ("(&'a u8,*const P)", vec![2]),
        ("(&'a u8,*const T)", vec![1]),
    ] {
        let source = format!(
            "trait A<P>{{type Out:?Sized;}}struct S<'a,P,T:A<P>+?Sized>{{prefix:{prefix},tail:T::Out}}"
        );
        let (_, facts) = semantic::analyze(&source)?;
        let item = facts
            .aggregates
            .iter()
            .next()
            .ok_or("missing test declaration")?
            .item;
        let mut rules = Rules::default();
        assert_eq!(
            rules.parameter(&facts.types, &facts.aggregates, item)?,
            None
        );
        assert_eq!(
            rules.projected_parameters(&facts.types, &facts.aggregates, item)?,
            expected
        );
        let before = rules.steps;
        assert_eq!(
            rules.projected_parameters(&facts.types, &facts.aggregates, item)?,
            expected
        );
        assert_eq!(rules.steps, before);
    }
    Ok(())
}

#[test]
fn projected_fields_follow_one_tail_without_rechecking_marker_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [4, 8, 16, 32] {
        let mut source = String::from(
            "trait A{type Out:?Sized;}impl<T,const N:usize>A for [T;N]{type Out=[T;N];}impl<T>A for [T]{type Out=[T];}struct W<T:A+?Sized>{tail:T::Out}impl<T:A+?Sized>A for W<T>{type Out=W<T>;}type L0=[u8;2];type R0=[u8];",
        );
        for index in 1..=count {
            write!(
                source,
                "type L{index}=W<L{}>;type R{index}=W<R{}>;",
                index - 1,
                index - 1
            )?;
        }
        write!(source, "fn convert(s:&L{count})->&R{count}{{s}}")?;
        let (_, facts) = semantic::analyze(&source)?;
        let work = &facts.body_work;
        eprintln!(
            "wrappers={count} inputs={} fields={} inference={}",
            work.unsizing_inputs, work.unsizing_fields, work.inference_steps
        );
        assert!(work.unsizing_inputs <= count * 12 + 16);
        assert!(work.unsizing_fields <= count * 4);
        assert!(work.unsizing_fields >= count);
    }
    Ok(())
}
