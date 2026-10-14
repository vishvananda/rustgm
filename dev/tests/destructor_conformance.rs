//! Generic Drop conformance and inherited method proofs use independently declared outcomes.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
#[path = "helpers/template_check.rs"]
mod template;

#[test]
fn implicit_field_receivers_are_ready_before_generic_body_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for (index, source) in [
        "struct S<T>(T);fn f<T:Copy>(s:&S<T>)->T{s.0}",
        "struct S<T>(T);fn f<T:Copy>(s:&&S<T>)->T{s.0}",
        "struct S<const N:usize>([u8;N]);fn f<const N:usize>(s:&mut S<N>){s.0=[2;N];}",
    ]
    .into_iter()
    .enumerate()
    {
        rustc::qualify(&format!("implicit-field-receiver-{index}"), source, true)?;
        assert_eq!(template::check(source)?.len(), 1);
    }
    Ok(())
}

#[test]
fn implicit_receiver_preparation_preserves_canonical_type_order() -> Result<(), String> {
    let source = "struct S<T>{value:T,extra:(T,T)}fn f<A,B,C,D>(a:&S<A>,b:&S<B>,c:&S<C>,d:&S<D>){let a=&a.value;let b=&b.value;let c=&c.value;let d=&d.value;}";
    let snapshot = || -> Result<Vec<_>, String> {
        let (_, facts) = semantic::analyze(source)?;
        (0..facts.types.len())
            .map(|index| {
                let index = u32::try_from(index).map_err(|e| e.to_string())?;
                facts
                    .types
                    .kind(rustgm::types::TypeId(index))
                    .ok_or_else(|| "missing canonical type".into())
            })
            .collect()
    };
    let expected = snapshot()?;
    for _ in 0..24 {
        assert_eq!(snapshot()?, expected);
    }
    Ok(())
}

#[test]
fn method_parameters_retain_the_impl_owner_and_reordered_field_identity()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{Kind, arguments::Argument};
    let source = "struct S<T,U:Copy>(T,U);impl<A:Copy,B> Drop for S<B,A>{fn drop(&mut self){let x=self.1;let y=self.1;let r=&self.0;}}";
    rustc::qualify("inherited-drop-identities", source, true)?;
    template::check(source)?;
    let (_, facts) = semantic::analyze(source)?;
    let implementation = facts
        .destructors
        .implementations()
        .first()
        .ok_or("missing Drop impl")?;
    assert!(
        !facts
            .generics
            .parameters
            .contains_key(&implementation.method)
    );
    let parameters = facts
        .generics
        .parameters_in_scope(implementation.method)
        .map(|p| p.argument)
        .collect::<Vec<_>>();
    assert_eq!(parameters.len(), 2);
    for &parameter in &parameters {
        let Argument::Type(ty) = parameter else {
            return Err("missing type parameter".into());
        };
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err("missing rigid parameter".into());
        };
        assert_eq!(
            facts
                .types
                .parameter(id)
                .ok_or("missing parameter identity")?
                .owner
                .0,
            implementation.self_definition.0
        );
    }
    let fields = facts
        .aggregates
        .shape(&facts.types, implementation.target)?
        .variant(0)
        .ok_or("missing fields")?
        .fields;
    let fields = facts.types.list(fields).ok_or("missing field list")?;
    assert_eq!(
        parameters,
        vec![Argument::Type(fields[1]), Argument::Type(fields[0])]
    );
    Ok(())
}

#[test]
fn wide_drop_schemas_preserve_linear_canonical_storage() -> Result<(), Box<dyn std::error::Error>> {
    let count = 256;
    let declared = (0..count)
        .map(|i| format!("T{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let actual = (0..count)
        .rev()
        .map(|i| format!("T{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "#![recursion_limit=\"2048\"]struct S<{declared}>({declared});impl<{declared}> Drop for S<{actual}>{{fn drop(&mut self){{}}}}"
    );
    rustc::qualify("wide-drop-schema", &source, true)?;
    let (report, facts) = semantic::analyze(&source)?;
    assert_eq!(
        rustgm::body::templates::build(&report, &facts)?
            .bodies
            .len(),
        1
    );
    assert!(facts.types.len() < count * 20);
    assert!(facts.aggregates.instantiation_steps() < count * 32);
    Ok(())
}

#[test]
fn generic_destructors_cover_their_nominal_types_and_prove_their_bodies()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, source) in CASES {
        rustc::qualify(&format!("generic-drop-{name}"), source, accepted)?;
        let result = template::check(source);
        if result.is_ok() != accepted {
            failures.push(format!(
                "{name}: wanted {accepted}; {}",
                result.err().map_or_else(String::new, |e| e.to_string())
            ));
        } else if let Ok(analyses) = result {
            assert_eq!(
                analyses.len(),
                1,
                "{name}: missing inherited generic method template"
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

const CASES: &[(&str, bool, &str)] = &[
    (
        "mixed-permutation",
        true,
        "struct S<'a,'b,T,U,const N:usize,const M:usize>(&'a T,&'b U,[u8;N],[u16;M]);impl<'x,'y,A,B,const X:usize,const Y:usize> Drop for S<'y,'x,B,A,Y,X> where A:'x,B:'y{fn drop(&mut self){let a=self.0;let b=self.1;let c=&self.2;let d=&self.3;}}",
    ),
    (
        "anonymous-mixed-copy",
        true,
        "struct S<'a,T:Copy,const N:usize>(&'a T,[u8;N]);impl<const N:usize,T:Copy> Drop for S<'_,T,N>{fn drop(self:&mut Self){let value=*self.0;let again=value;self.1=[0;N];}}",
    ),
    (
        "unused-lifetime-outlives-path",
        true,
        "struct S<'a,T>(&'a T);impl<'a,'x,T> Drop for S<'a,T>where T:'x,'x:'a{fn drop(&mut self){}}",
    ),
    (
        "unused-lifetime-strengthening",
        false,
        "struct S<'a,T>(&'a T);impl<'a,'x,T> Drop for S<'a,T>where T:'x,'x:'static{fn drop(&mut self){}}",
    ),
    (
        "permuted-outlives-identity",
        false,
        "struct S<'a,T,U>(&'a T,U);impl<'a,A,B> Drop for S<'a,B,A>where A:'a{fn drop(&mut self){}}",
    ),
    (
        "callable-bound-local-lifetime",
        true,
        "struct S<T>(T);impl<T> Drop for S<T>where (for<'a> fn(&'a T),):Copy{fn drop(&mut self){}}",
    ),
    (
        "callable-self-annotation",
        true,
        "struct S<'a,T>(&'a T);impl<T> Drop for S<'_,T>{fn drop(&mut self){let f:for<'x> fn(&'x Self);}}",
    ),
    (
        "receiver-local-lifetime",
        true,
        "struct S<'a,T>(&'a T);impl<'a,T> Drop for S<'a,T>{fn drop<'r>(&'r mut self){let r=self.0;}}",
    ),
    (
        "type",
        true,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "const",
        true,
        "struct S<const N:usize>([u8;N]);impl<const N:usize> Drop for S<N>{fn drop(&mut self){}}",
    ),
    (
        "mixed",
        true,
        "struct S<'a,T,const N:usize>(&'a T,[u8;N]);impl<'a,T,const N:usize> Drop for S<'a,T,N>{fn drop(&mut self){}}",
    ),
    (
        "permuted",
        true,
        "struct S<T,U>(T,U);impl<U,T> Drop for S<T,U>{fn drop(&mut self){}}",
    ),
    (
        "alias-permuted",
        true,
        "struct S<T,U>(T,U);type A<T,U>=S<U,T>;impl<T,U> Drop for A<T,U>{fn drop(&mut self){}}",
    ),
    (
        "duplicate-type",
        false,
        "struct S<T,U>(T,U);impl<T> Drop for S<T,T>{fn drop(&mut self){}}",
    ),
    (
        "closed-type",
        false,
        "struct S<T>(T);impl Drop for S<u8>{fn drop(&mut self){}}",
    ),
    (
        "wrapped-type",
        false,
        "struct S<T>(T);impl<T> Drop for S<[T;1]>{fn drop(&mut self){}}",
    ),
    (
        "closed-const",
        false,
        "struct S<const N:usize>;impl Drop for S<0>{fn drop(&mut self){}}",
    ),
    (
        "duplicate-const",
        false,
        "struct S<const N:usize,const M:usize>;impl<const N:usize> Drop for S<N,N>{fn drop(&mut self){}}",
    ),
    (
        "block-const",
        true,
        "struct S<const N:usize>;impl<const N:usize> Drop for S<{N}>{fn drop(&mut self){}}",
    ),
    (
        "copy-extra",
        false,
        "struct S<T>(T);impl<T:Copy> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "copy-exact",
        true,
        "struct S<T:Copy>(T);impl<T:Copy> Drop for S<T>{fn drop(&mut self){let x=self.0;let _=x;}}",
    ),
    (
        "copy-missing",
        false,
        "struct S<T:Copy>(T);impl<T> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "clone-implied",
        true,
        "struct S<T:Copy>(T);impl<T:Copy+Clone> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "unused-type",
        false,
        "struct S;impl<T> Drop for S{fn drop(&mut self){}}",
    ),
    (
        "unused-const",
        false,
        "struct S;impl<const N:usize> Drop for S{fn drop(&mut self){}}",
    ),
    (
        "unused-lifetime",
        true,
        "struct S<T>(T);impl<'a,T> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "outlives-explicit",
        true,
        "struct S<'a,T:'a>(&'a T);impl<'a,T:'a> Drop for S<'a,T>{fn drop(&mut self){}}",
    ),
    (
        "outlives-implied",
        true,
        "struct S<'a,T>(&'a T);impl<'a,T:'a> Drop for S<'a,T>{fn drop(&mut self){}}",
    ),
    (
        "outlives-extra",
        false,
        "struct S<T>(T);impl<T:'static> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "explicit-receiver",
        true,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop(self:&mut Self){}}",
    ),
    (
        "method-type",
        false,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop<U>(&mut self){}}",
    ),
    (
        "method-extra-bound",
        false,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self)where T:Copy{}}",
    ),
    (
        "impl-default",
        false,
        "struct S<T>(T);impl<T=u8> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "body-forbidden-move",
        false,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self){let _x=self.0;}}",
    ),
    (
        "body-const-repeat",
        true,
        "struct S<const N:usize>([u8;N]);impl<const N:usize> Drop for S<N>{fn drop(&mut self){let a=[3u8;N];self.0=a;}}",
    ),
    (
        "nested-function-capture",
        false,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self){fn f(x:T){}}}",
    ),
    (
        "method-implied-copy",
        true,
        "struct S<T:Copy>(T);impl<T:Copy> Drop for S<T>{fn drop(&mut self)where T:Copy{let x=self.0;let _=x;}}",
    ),
    (
        "method-implied-clone",
        true,
        "struct S<T:Copy>(T);impl<T:Copy> Drop for S<T>{fn drop(&mut self)where T:Clone{}}",
    ),
    (
        "impl-implied-type-sized",
        true,
        "struct S<T>(T);impl<T:?Sized> Drop for S<T>where T:Sized{fn drop(&mut self){}}",
    ),
    (
        "swapped-consts",
        true,
        "struct S<const N:usize,const M:usize>;impl<const M:usize,const N:usize> Drop for S<N,M>{fn drop(&mut self){}}",
    ),
    (
        "bool-const",
        true,
        "struct S<const B:bool>;impl<const B:bool> Drop for S<B>{fn drop(&mut self){}}",
    ),
    (
        "char-const",
        true,
        "struct S<const C:char>;impl<const C:char> Drop for S<C>{fn drop(&mut self){}}",
    ),
    (
        "derived-copy",
        false,
        "#[derive(Clone,Copy)]struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "duplicate-impl",
        false,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self){}}impl<U> Drop for S<U>{fn drop(&mut self){}}",
    ),
    (
        "nominal-outlives-static",
        true,
        "struct S<T:'static>(T);impl<T:'static> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "method-outlives-extra",
        false,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self)where T:'static{}}",
    ),
    (
        "nominal-composite",
        true,
        "struct S<T>where [T;0]:Copy{v:T}impl<T> Drop for S<T>where [T;0]:Copy{fn drop(&mut self){}}",
    ),
    (
        "composite-extra-parameter",
        false,
        "struct S<T>where [T;0]:Copy{v:T}impl<T:Copy> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "unsized-pointer",
        true,
        "struct S<T:?Sized>{v:*const T}impl<T:?Sized> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "sized-pointer-extra",
        false,
        "struct S<T:?Sized>{v:*const T}impl<T> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "unsized-function",
        true,
        "struct S<T:?Sized>{v:fn(*const T)}impl<T:?Sized> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "sized-function-extra",
        false,
        "struct S<T:?Sized>{v:fn(*const T)}impl<T> Drop for S<T>{fn drop(&mut self){}}",
    ),
    (
        "anonymous-impl",
        true,
        "struct S<'a,T>(&'a T);impl<T> Drop for S<'_,T>{fn drop(self:&mut Self){}}",
    ),
    (
        "anonymous-self-bound",
        true,
        "struct S<'a,T>(&'a T);impl<T> Drop for S<'_,T>where Self:Sized{fn drop(&mut self){}}",
    ),
    (
        "method-receiver-bound",
        false,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop<'a>(&'a mut self)where T:'a{}}",
    ),
    (
        "method-receiver-copy",
        false,
        "struct S<T>(T);impl<T> Drop for S<T>{fn drop<'a>(&'a mut self)where &'a T:Copy{}}",
    ),
];
