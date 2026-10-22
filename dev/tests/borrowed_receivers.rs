//! Implicit receiver places retain their selected, instantiated storage roots.
use crate::native_references as native;
use crate::semantic;
use crate::source_cases;

const CASES: &[(&str, bool, &str)] = &[
    (
        "shared",
        true,
        "struct S<T>(T);impl<T:Copy>S<T>{fn get(&self)->T{self.0}}fn f(s:&S<u8>)->u8{s.get()}",
    ),
    (
        "mutable",
        true,
        "struct S<T>(T);impl<T:Copy>S<T>{fn set(&mut self,x:T){self.0=x;}}fn f(s:&mut S<u8>){s.set(2);}",
    ),
    (
        "nested-reference",
        true,
        "struct S<T>(T);impl<T:Copy>S<T>{fn get(&self)->T{self.0}}fn f(s:&&S<u8>)->u8{s.get()}",
    ),
    (
        "generic-caller",
        true,
        "struct S<T>(T);impl<T:Copy>S<T>{fn get(&self)->T{self.0}}fn f<T:Copy>(s:&S<T>)->T{s.get()}",
    ),
    (
        "reference-field",
        true,
        "struct S<T>(T);impl<T:Copy>S<T>{fn get(&self)->T{self.0}}fn f<'a>(s:&S<&'a u8>)->&'a u8{s.get()}",
    ),
    (
        "enum",
        true,
        "enum E<T>{V(T)}impl<T:Copy>E<T>{fn get(&self)->T{match self{E::V(x)=>*x}}}fn f(s:&E<u8>)->u8{s.get()}",
    ),
    (
        "generic-const-owner",
        true,
        "struct S<T,const N:usize>([T;N]);impl<T:Copy,const N:usize>S<T,N>{fn get(&self)->T{self.0[0]}}fn f(s:&S<u8,2>)->u8{s.get()}",
    ),
    (
        "shared-to-mutable",
        false,
        "struct S<T>(T);impl<T:Copy>S<T>{fn set(&mut self,x:T){self.0=x;}}fn f(s:&S<u8>){s.set(2);}",
    ),
    (
        "conflicting-field-borrow",
        false,
        "struct S<T>(T);impl<T:Copy>S<T>{fn set(&mut self,x:T){self.0=x;}}fn f(s:&mut S<u8>)->u8{let r=&s.0;s.set(2);*r}",
    ),
    (
        "mismatched-input",
        false,
        "struct S<T>(T);impl<T:Copy>S<T>{fn set(&mut self,x:T){self.0=x;}}fn f(s:&mut S<u8>){s.set(2u16);}",
    ),
];

#[test]
fn implicit_receiver_storage_matches_rustc() -> Result<(), Box<dyn std::error::Error>> {
    source_cases::check("method-storage", CASES)
}

#[test]
fn repeated_receiver_roots_share_completed_field_instances() -> Result<(), String> {
    use rustgm::types::Kind;
    for fields in [1, 32, 128] {
        let mut first = None;
        for count in [1, 32, 256] {
            let source = format!(
                "struct S<T>({});impl<T:Copy>S<T>{{fn get(&self)->T{{self.0}}}}fn f(s:&S<u8>){{{}}}",
                "T,".repeat(fields),
                "s.get();".repeat(count)
            );
            let (_, facts) = semantic::analyze(&source)?;
            assert_eq!(facts.methods.calls.len(), count);
            let mut selected = None;
            for call in facts.methods.calls.values() {
                let Some(Kind::Reference { referent, .. }) =
                    call.borrow.and_then(|ty| facts.types.kind(ty))
                else {
                    return Err("missing selected receiver borrow".into());
                };
                assert_eq!(referent, *selected.get_or_insert(referent));
                facts.aggregates.shape(&facts.types, referent)?;
            }
            let steps = facts.aggregates.instantiation_steps();
            let baseline = *first.get_or_insert(steps);
            // Fresh borrow regions can add constant work per call. Increasing
            // callers must not repeat substitution of the complete field list.
            assert!(
                steps <= baseline + 4 * count,
                "{fields} fields, {count} calls: {steps}"
            );
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn prepared_receivers_keep_runtime_accesses() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pa24-pa25-work/native-method-storage",
        &[
            (
                "shared",
                "struct S<T>(T);impl<T:Copy>S<T>{fn get(&self)->T{self.0}}fn f(s:&S<u8>)->u8{s.get()}fn main(){check(f(&S(3))==3);}",
                false,
            ),
            (
                "mutable",
                "struct S<T>(T);impl<T:Copy>S<T>{fn set(&mut self,x:T){self.0=x;}}fn f(s:&mut S<u8>){s.set(7);}fn main(){let mut s=S(3);f(&mut s);check(s.0==7);}",
                false,
            ),
            (
                "generic",
                "struct S<T>(T);impl<T:Copy>S<T>{fn get(&self)->T{self.0}}fn f<T:Copy>(s:&S<T>)->T{s.get()}fn main(){check(f(&S(5u8))==5);}",
                false,
            ),
        ],
    )
}
