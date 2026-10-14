//! Array bounds and runtime const operands survive symbolic body proofs unchanged.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
#[path = "helpers/template_check.rs"]
mod template;
use rustgm::{
    body::ir::{ProjectionKind, Rvalue},
    types::{Kind, TypeId, length::Length},
};

const CASES: &[(&str, bool, &str)] = &[
    (
        "read",
        true,
        "fn f<T:Copy,const N:usize>(a:[T;N],i:usize)->T{a[i]}",
    ),
    (
        "reference",
        true,
        "fn f<'a,T,const N:usize>(a:&'a [T;N],i:usize)->&'a T{&a[i]}",
    ),
    (
        "write",
        true,
        "fn f<const N:usize>(a:&mut [u8;N],i:usize){a[i]=3;a[i]+=1;}",
    ),
    (
        "nested",
        true,
        "fn f<T:Copy,const N:usize,const M:usize>(a:&[[T;N];M],i:usize,j:usize)->T{a[i][j]}",
    ),
    (
        "inferred-bound",
        true,
        "struct S<T,const N:usize>([T;N]);fn f<T:Copy>(x:T)->T{let s:S<T,_>=S([x;3]);s.0[2]}",
    ),
    (
        "noncopy-move",
        false,
        "fn f<T,const N:usize>(a:[T;N])->T{a[0]}",
    ),
    (
        "overlapping-borrow",
        false,
        "fn f<const N:usize>(a:&mut [u8;N],i:usize){let r=&mut a[i];a[i]=2;*r=3;}",
    ),
    (
        "immutable-write",
        false,
        "fn f<const N:usize>(a:&[u8;N]){a[0]=2;}",
    ),
    ("usize-operand", true, "fn f<const N:usize>()->usize{N+1}"),
    ("bool-operand", true, "fn f<const B:bool>()->bool{!B}"),
    ("char-operand", true, "fn f<const C:char>()->char{C}"),
    ("signed-operand", true, "fn f<const N:i8>()->i8{N}"),
    (
        "const-index",
        true,
        "fn f<const N:usize,const I:usize>(a:[u8;N])->u8{a[I]}",
    ),
];

#[test]
fn operand_programs_match_rustc_and_complete_symbolic_cleanup()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, source) in CASES {
        rustc::qualify(&format!("symbolic-operands-{name}"), source, accepted)?;
        let result = template::check(source);
        if result.is_ok() != accepted {
            failures.push(format!(
                "{name}: expected {accepted}, {}",
                result.err().map_or_else(String::new, |e| e.to_string())
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn projections_keep_their_exact_const_parameter_and_inferred_bounds()
-> Result<(), Box<dyn std::error::Error>> {
    for (source, lengths) in [
        (
            "fn f<const N:usize,const M:usize>(a:&[[u8;N];M],i:usize,j:usize)->u8{a[i][j]}",
            vec![1, 0],
        ),
        (
            "struct S<T,const N:usize>([T;N]);fn f<T:Copy>(x:T)->T{let s:S<T,_>=S([x;3]);s.0[2]}",
            vec![3],
        ),
    ] {
        let (report, facts) = semantic::analyze(source)?;
        let templates = rustgm::body::templates::build(&report, &facts)?;
        let body = &templates.bodies.first().ok_or("missing template")?.body;
        let mut actual = Vec::new();
        for projection in &body.projections {
            let length = match projection.kind {
                ProjectionKind::Index { length, .. } => Length::Known(length),
                ProjectionKind::IndexParameter { length, .. } => Length::Parameter(length),
                _ => continue,
            };
            actual.push(match length {
                Length::Known(value) => value,
                Length::Parameter(id) => {
                    let Some(rustgm::types::arguments::ConstKind::Parameter(id)) =
                        facts.types.const_kind(id)
                    else {
                        return Err("lost const identity".into());
                    };
                    u64::from(facts.types.parameter(id).ok_or("missing parameter")?.index)
                }
            });
        }
        assert_eq!(actual, lengths);
    }
    Ok(())
}

#[test]
fn runtime_const_operands_use_canonical_ids_instead_of_named_constant_lookups()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, facts) = semantic::analyze("fn f<const N:usize>()->usize{N+N}")?;
    let expected = *facts
        .generics
        .constants
        .values()
        .next()
        .ok_or("missing const argument")?;
    let templates = rustgm::body::templates::build(&report, &facts)?;
    let body = &templates.bodies.first().ok_or("missing template")?.body;
    let values = body
        .values
        .iter()
        .filter_map(|v| match v.kind {
            Rvalue::ConstArgument(id) => Some(id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(values, vec![expected, expected]);
    assert!(
        body.values
            .iter()
            .all(|v| !matches!(v.kind, Rvalue::Constant(_)))
    );
    Ok(())
}

#[test]
fn bound_substitution_keeps_foreign_parameters_and_checks_scalar_kinds() -> Result<(), &'static str>
{
    use rustgm::types::{
        ItemId, Primitive, Store,
        arguments::{Argument, ConstKind, Constant},
        parameters::{Parameter, ParameterKind},
        substitute::Bound,
    };
    let mut store = Store::default();
    let owner = ItemId(7);
    let usize_ = TypeId::primitive(Primitive::Usize);
    let parameter = store.intern_parameter(Parameter {
        owner,
        index: 0,
        kind: ParameterKind::Const { ty: usize_ },
    })?;
    let id = store.intern_const(ConstKind::Parameter(parameter))?;
    let array = store.intern(Kind::ArrayTerm {
        element: TypeId::UNIT,
        length: id,
    })?;
    let length = Length::of(&store, array)?;
    assert_eq!(length, Length::Parameter(id));
    assert_eq!(length.known(), None);
    for bits in [0, 3, u128::from(u64::MAX)] {
        let argument = store.intern_constant(Constant { ty: usize_, bits })?;
        let args = store.intern_arguments(&[Argument::Const(argument)])?;
        let mut substitution = Bound::joint(&store, owner, args)?;
        assert_eq!(
            length.substitute(&mut store, &mut substitution)?.known(),
            u64::try_from(bits).ok()
        );
        let mut foreign = Bound::joint(&store, ItemId(8), args)?;
        assert_eq!(length.substitute(&mut store, &mut foreign)?, length);
    }
    let wrong = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Bool),
        bits: 1,
    })?;
    let args = store.intern_arguments(&[Argument::Const(wrong)])?;
    let mut substitution = Bound::joint(&store, owner, args)?;
    assert!(length.substitute(&mut store, &mut substitution).is_err());
    assert!(Length::of(&store, TypeId::UNIT).is_err());
    Ok(())
}

#[test]
fn const_argument_values_preserve_bool_char_and_signed_integer_bits() -> Result<(), &'static str> {
    use rustgm::{
        sema::{model::Scalar, values::Constant},
        types::{
            Primitive, Store,
            arguments::{ConstId, Constant as Argument},
        },
    };
    let mut store = Store::default();
    for (primitive, bits, expected) in [
        (Primitive::Bool, 1, Scalar::Boolean(true)),
        (Primitive::Char, 0x1f600, Scalar::Character('😀')),
        (Primitive::I8, 255, Scalar::Integer(255)),
        (Primitive::U128, u128::MAX, Scalar::Integer(u128::MAX)),
    ] {
        let id = store.intern_constant(Argument {
            ty: TypeId::primitive(primitive),
            bits,
        })?;
        assert_eq!(
            Constant::argument(&store, id)?
                .ok_or("missing constant")?
                .as_scalar()
                .map_err(rustgm::sema::values::Failure::message)?,
            expected
        );
    }
    assert!(Constant::argument(&store, ConstId(u32::MAX)).is_err());
    Ok(())
}
