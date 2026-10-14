//! Storage recursion is about owned parameter flow, not repeated item names or all arguments.
#[path = "helpers/qualify.rs"]
mod rustc;
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, Store, TypeId,
    aggregate::{
        self, Definitions, Repr, Variant,
        finiteness::{Error, Storage},
    },
    arguments::Argument,
    parameters::{Parameter, ParameterKind},
};
use std::fmt::Write as _;
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[derive(Default)]
struct Fixture {
    types: Store,
    definitions: Definitions,
    schemas: Vec<(ItemId, GenericArgs)>,
}
impl Fixture {
    fn parameter(&mut self, owner: u32) -> Result<TypeId, &'static str> {
        let id = self.types.intern_parameter(Parameter {
            owner: ItemId(owner),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        self.types.intern(Kind::Parameter(id))
    }
    fn nominal(&mut self, owner: u32, parameter: Option<TypeId>) -> Result<TypeId, &'static str> {
        let arguments: Vec<_> = parameter.into_iter().map(Argument::Type).collect();
        let arguments = self.types.intern_arguments(&arguments)?;
        self.types.intern(Kind::Adt(ItemId(owner), arguments))
    }
    fn define(
        &mut self,
        owner: u32,
        parameter: Option<TypeId>,
        fields: &[TypeId],
    ) -> Result<(), &'static str> {
        let arguments: Vec<_> = parameter.into_iter().map(Argument::Type).collect();
        self.schemas
            .push((ItemId(owner), self.types.intern_arguments(&arguments)?));
        let fields = self.types.intern_list(fields)?;
        self.definitions.insert(
            ItemId(owner),
            aggregate::Kind::Struct,
            Repr::default(),
            &[Variant {
                fields,
                discriminant: 0,
            }],
        )
    }
    fn infer(&self) -> Result<Storage, Error> {
        Storage::infer(&self.types, &self.definitions, &self.schemas)
    }
}

#[test]
fn nested_storage_requirements_use_both_selected_parameter_positions() -> TestResult {
    for a in [false, true] {
        for b in [false, true] {
            let mut f = Fixture::default();
            for (owner, stored) in [(0, a), (1, b)] {
                let t = f.parameter(owner)?;
                let field = if stored {
                    t
                } else {
                    f.types.intern(Kind::Pointer {
                        mutable: false,
                        pointee: t,
                    })?
                };
                f.define(owner, Some(t), &[field])?;
            }
            let t = f.parameter(2)?;
            let inner = f.nominal(1, Some(t))?;
            let outer = f.nominal(0, Some(inner))?;
            f.define(2, Some(t), &[outer])?;
            let before = f.types.len();
            for _ in 0..3 {
                let storage = f.infer().map_err(|e| format!("{e:?}"))?;
                assert_eq!(storage.arguments(ItemId(2)), Some([a && b].as_slice()));
                f.schemas.rotate_left(1);
            }
            let s = f.nominal(3, None)?;
            let field = f.nominal(2, Some(s))?;
            f.define(3, None, &[field])?;
            assert_eq!(f.infer().is_err(), a && b);
            assert_eq!(f.definitions.instantiation_steps(), 0);
            assert_eq!(f.types.len(), before + 2);
        }
    }
    Ok(())
}

#[test]
fn zero_arrays_require_alignment_but_pointers_stop_the_storage_walk() -> TestResult {
    for (wrapper, recursive) in [(0, true), (1, false), (2, false)] {
        let mut f = Fixture::default();
        let t = f.parameter(0)?;
        let field = match wrapper {
            0 => f.types.intern(Kind::Array {
                element: t,
                length: 0,
            })?,
            1 => f.types.intern(Kind::Pointer {
                mutable: false,
                pointee: t,
            })?,
            _ => {
                let parameters = f.types.intern_list(&[t])?;
                f.types.intern(Kind::Function {
                    parameters,
                    result: t,
                    bound_regions: 0,
                    unsafe_: false,
                })?
            }
        };
        f.define(0, Some(t), &[field])?;
        let s = f.nominal(1, None)?;
        let field = f.nominal(0, Some(s))?;
        f.define(1, None, &[field])?;
        assert_eq!(f.infer().is_err(), recursive);
        assert_eq!(f.definitions.instantiation_steps(), 0);
    }
    Ok(())
}

#[test]
fn finite_repeated_items_and_growing_recursive_templates_are_distinct() -> TestResult {
    let mut f = Fixture::default();
    let t = f.parameter(0)?;
    f.define(0, Some(t), &[t])?;
    let mut ty = TypeId::primitive(Primitive::U8);
    for _ in 0..4096 {
        ty = f.nominal(0, Some(ty))?;
    }
    f.define(1, None, &[ty])?;
    let before = f.types.len();
    let storage = f.infer().map_err(|e| format!("{e:?}"))?;
    assert!(storage.steps < 50_000, "{} steps", storage.steps);
    assert_eq!(f.types.len(), before);
    assert_eq!(f.definitions.instantiation_steps(), 0);
    let mut g = Fixture::default();
    let t = g.parameter(0)?;
    let array = g.types.intern(Kind::Array {
        element: t,
        length: 1,
    })?;
    let next = g.nominal(0, Some(array))?;
    g.define(0, Some(t), &[next, t])?;
    assert!(matches!(g.infer(), Err(Error::Recursive(ItemId(0)))));
    assert_eq!(g.definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn long_forward_storage_dependencies_propagate_once() -> TestResult {
    let mut f = Fixture::default();
    for owner in 0..4096 {
        let t = f.parameter(owner)?;
        let field = if owner == 4095 {
            t
        } else {
            f.nominal(owner + 1, Some(t))?
        };
        f.define(owner, Some(t), &[field])?;
    }
    for _ in 0..2 {
        let storage = f.infer().map_err(|e| format!("{e:?}"))?;
        for owner in 0..4096 {
            assert_eq!(storage.arguments(ItemId(owner)), Some([true].as_slice()));
        }
        assert!(storage.steps < 100_000, "{} steps", storage.steps);
        f.schemas.reverse();
    }
    Ok(())
}

#[test]
fn storage_results_agree_with_independent_rustc() -> TestResult {
    for a in [false, true] {
        for b in [false, true] {
            let mut source = String::new();
            for (name, stored) in [("A", a), ("B", b)] {
                write!(
                    source,
                    "struct {name}<T>{{v:{}T}}",
                    if stored { "" } else { "*const " }
                )?;
            }
            source.push_str("struct C<T>{v:A<B<T>>}struct S{v:C<S>}");
            rustc::qualify(&format!("storage-products-{a}-{b}"), &source, !(a && b))?;
        }
    }
    for (name, source, accepted) in [
        ("finite", "struct S<T>{v:T}struct W{v:S<S<u8>>}", true),
        ("growing", "struct S<T>{v:S<[T;1]>,t:T}", false),
        ("zero", "struct Wrap<T>{v:[T;0]}struct S{v:Wrap<S>}", false),
        (
            "function",
            "struct Wrap<T>{v:fn(T)->T}struct S{v:Wrap<S>}",
            true,
        ),
    ] {
        rustc::qualify(&format!("storage-{name}"), source, accepted)?;
    }
    Ok(())
}
