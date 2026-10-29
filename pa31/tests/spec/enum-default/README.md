# Source enum Default and concrete Self constructors

This planned PA31 cluster connects PA26's real Default derive to source-owned
macro/trait declarations and ordinary constructor proof. Sidecar contracts freeze
inputs and predictions before observation; no assignment manifest is activated.

`upstream-abi.rs` preserves the unchanged original `Abi` declaration, attributes,
three variants and `#[derive(Debug, Default)]` from pinned Rust88d9
`library/core/src/mem/type_info.rs`, bytes10239–10543. `abi-expansion.rs` composes
that exact fragment with the existing genuine source macro declarations;
`abi-rust.rs` independently observes the original with Rust. The recipe source is
[the pinned compiler Default derive](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_builtin_macros/src/deriving/default.rs),
SHA256 `96a7af1c5af9047e1160fe4c1fd2785b12cf2199ca47c8cc502071447c0b5446`.
Rust attribution and MIT/Apache-2.0 licensing remain governed by root NOTICE.

The actual selected Default header must declare its `default` helper. Selection
uses configured active syntax and real owner/effect/capture identities, including
cfg_attr. Exactly one word marker selects a unit variant. Duplicate/malformed,
missing, misplaced, multiple, nonunit and non_exhaustive-variant markers reject.
A non_exhaustive enum itself is supported. The emitted method returns the pinned
`Self::Variant` expression; declared type/lifetime/const parameters and explicit
bounds/where clauses survive, while enum Default adds no T:Default premise.
Struct Default retains its existing field calls and generated type bounds.

`generic-source.rs` uses the existing genuine NoCore foundation with a payload
that has no Default implementation. Its mixed type/lifetime/const declaration,
parameter defaults, explicit where clause and configured variant selection reach
real source checking, instances, LowIR and native `7`. `self-source.rs` separately
uses user-written inherent and trait impl bodies with concrete `Self::Empty`.
The semantic port reuses existing alias prefix/enum member/constructor kernels;
actual owner, privacy and generic substitution remain authoritative. Separate
missing, wrong-owner, nonunit bare value, wrong generic application, illegal
arguments, abstract Self and non-enum opponents remain negative. Existing enum
associated constants/methods retain their own lookup path.

The first frozen Rust packet matched22/23 command predictions. Its mistaken
const positive omitted the real `const_default` library request; both its failed
receipt and input remain intact. `const-successor.json` freezes only the corrected
pair, which passes and preserves `const impl<T>` without T:Default. The separate
Self packet matches all14 observations, including the genuine native `7`.
Neither a successful expansion nor the const header alone grants const API use.

Final candidate qualification is six focused private controls, including the old
tuple/record Default source consumer, and two actual LowIR-to-object/C-native
programs matching independent exact-source Rust output. Helper mutation, cfg
removal and clearing a required stored choice invalidate replay. Exhausted payment
publishes no derive. Current cold sizes are Site248/Transaction224/Choice24;
Transaction previously occupied200. Dense syntax Node36 and Body456 stay unchanged.
Every enum receipt rechecks its actual local item with paid temporary scratch;
warm work/storage is explicitly nonzero, not a whole-catalog scan. Exact measured
N8/32/128 values and source-bound receipts live in the worker handoff packet.

Complete Formatter/original Abi runtime, full original core/alloc/std artifacts,
nonunit record defaults with default_field_values, wider const/generic/metadata
transport and full parent assignment release gates remain Required. This local
source/native checkpoint does not publish a library artifact or close a PA.
