# Reached constant callback selection proposal

This is a source-only interface proposal, not an executable-support claim.
Runtime selection and the paired ordinary callable Copy/Clone correction are
qualified by the integrator. The original CTFE-positive source and its prior
explicit unsupported diagnostic remain unchanged. Production CTFE changes wait
for interface review and the integrator's release/performance boundary.

## Authority and selected values

The actual checked source call already proves ARG:Tuple, F:const FnOnce with
Output=RET, G:ordinary FnOnce with Output=RET, and its ownership/lifetime debts.
The intrinsic declaration supplies structural identity and relocation data.
Neither that data nor a cached function body supplies callback execution
permission. Only a reached constant Call chooses F. Ordinary execution keeps
choosing G through the qualified runtime adapter.

A private non-Copy `reached_select::Call<'code>` borrows the actual immutable
Code and retains its current Call ValueId, checked logical Select data and
evaluated actual F Instance. Operand IDs come from that Call's three-input
range. They do not come from the intrinsic declaration or another Call.

Its constructor takes `(facts, catalog, frame, position, evaluated_selector)`:

1. Validate the canonical caller Code index and checked block/statement range.
   The current statement must be Evaluate of this exact Rvalue::Call in a
   constant-executable Code. Snapshot identity includes source, owner/key, full
   instance and the public/internal signature pair.
2. Derive the raw logical selector Instance from the typed callee value. Require
   the actual registered source intrinsic and completed actual roles. Check the
   evaluated selector equals this Code's `callable_target(raw)` exactly once.
   Do not mistake the evaluated mapped signature for the caller's logical one.
3. Preserve the selector's public signature and its existing opened Call
   signature. The latter has exactly ARG/F/G inputs and RET result in the
   caller's namespace. Each actual operand type and the Call result must match
   those logical roots exactly.
4. Read all three already evaluated operand slots in source order. F must be an
   actual function Constant with the same source item as logical F. Its complete
   Instance is the Constant's existing mapped target; do not map it again or
   require equality with the constrained logical F Instance. Constrain, local
   reads and checked interprocedural returns preserve this value's origin.
5. Preserve the actual chosen callback's safe Rust public signature. Its body
   must pass the existing const eligibility, source policy, drop and actual
   member execution gates. Materialization validates its exact source/owner,
   complete Instance, public/internal signatures and actual canonical index.
   The caller's retained FnOnce equations justify the logical tuple/result
   relation; this adapter does not replay declarations, invent an Invoke, erase
   binders or re-prove conversion eligibility from rendered types.

The callback's public binder and its prepared body execution binder remain
distinct. Existing full-instance body materialization opens the actual callback
body. The selector adapter supplies no substitute zero-binder signature.

The witness binds the exact evaluated F slot. In particular a pending
FunctionBody(F) request or saved payment for another full F Instance is rejected
even when the source item is the same. Private frame corruption is not an
alternative producer of typed callable values.

## API and consumer changes proposed for review

| Owner | Proposed change | Existing consumer retained |
| --- | --- | --- |
| `compiler_items::select_calls` | Share the existing Select contract and canonical argument relocation preparation with actual CTFE Code publication. Remove blanket CTFE denial only when the reached adapter owns dispatch. | Actual registry/source header validation; runtime `prepare_select`/`source_select` |
| `const_eval::use_inputs::Inputs::needs` and its one `Engine::prove_body` caller | Rename the mutating publication scan to `prepare`; prepare all actual source-selector Call relocations during the existing charged forward scan, after body proof and before Code snapshot. Retain the actual caller Contract for selector sites. No selector body or F body is demanded here. | Existing CheckedInput/Inputs Row, source/owner/signature checks and retained storage accounting |
| New `const_eval::reached_select` | Construct the exact borrowed reached witness above; expose selected Instance, logical tuple/result and actual operand IDs only through checked accessors. | Actual Code/Frame values and `Catalog::validate_entry` |
| `const_eval::executed_calls` | Own registry dispatch to the selector adapter before any ordinary function-body lookup. Return unrelated, pending, or an actual callback entry plus spread arguments. | Existing source intrinsic dispatch owner; layout/needs_drop witnesses |
| `const_eval::values::Rvalue::Call` | Use the selector result before allocating the ordinary three-argument call vector; pass selected callback arguments to the existing child push and retain the original selector Call destination. | Existing suspend/retry of the current statement and ordinary return publication |
| `const_eval::function_calls::Form` | Add a Selected form carrying the raw selector Instance as identity data. Route F through the same common const/source/member gates and actual full-instance entry validation. Selected intrinsic F remains explicit unsupported; it never enters an intrinsic placeholder. | Existing Direct/Invoke paths and body cache |
| Cold `const_eval::use_calls::Site` and `inspect` | Add checked Direct/Selected form. A Selected site re-derives the exact selector Call and requires requested target equal the current evaluated F Instance. Existing code/block/position derives the Call ID and operands; do not retain duplicate fields. | The existing Reached/Continuation, exact caller/callee Rows and one-use paid_call slot |
| `pipeline::execution::prepare_reached_function` | Reuse FunctionBody(actual F). For Conditional F, `reached_const_call` recognizes the selected site, materializes that F, then pays its own Requirements in the actual caller Environment. | Existing `function_body`, `const_payments::prepare/pay`, available limits and continuation installation |

This proposal adds no State, Machine, Engine, Body, Code or Frame field. Site is
cold continuation identity data, not a permission-bearing hot instruction. It
adds no backend operation, generic proof engine, second payment map, per-value
origin object or speculative function-body walk.

## Tuple transfer and suspension

ARG is the already evaluated owned tuple Constant. Require its product shape,
no nominal variant and the exact logical tuple width. Charge width/storage
before allocating its spread vector. Once the actual callback entry is ready,
read fields in ordinal order through the existing constant product reader and
pass them to the ordinary child frame. The defining ownership proof decides
moves and Copy; copying a Constant handle is not a new language Copy proof.

Suspend with the existing FunctionBody(actual F) request. The retained parent
frame already owns all operand values, so resumption does not re-evaluate ARG,
F, G or their producing helper calls. Reinspect the current statement and F
before entry. Allocate no spread vector while preparation is pending. Return
the child's value through the original selector Call ValueId. No selector
placeholder frame is created, including for a body-bearing declaration.

Both unused callbacks retain their source obligations. CTFE does not demand G's
body merely because its item was evaluated. Helpers that produce G still run
when their operand expression is reached. Callback destruction/general callable
objects are outside this first profile; existing ordinary runtime drop behavior
is unchanged.

## Fresh payment and current frontiers

A Conditional F requires its own fresh Requirements/Payment, even if its Code
is cached. The current continuation binds the actual caller Environment and
selected F Row. Its saved site includes selector identity, F target and the
current frame/code/block/position. Existing reinspection, one-use consumption,
exact requirements Arc identity, environment identity and callee canonical index
checks apply on every cold and warm invocation. A selector's own permission or
payment cannot pay F. Failed preparation/payment publishes no usable success.

B's current `Eligibility::Unpaid` branch still rejects conditional callable
proof without a scoped continuation. Its default defining-context successor is
unqualified and supplies no execution permission. C does not bypass either
authority. The successful initial source target is an ordinary const function F
with closed empty ARG; ordinary const F with owned tuples is a related target.

Conditional empty-ARG F is valid Rust but remains pending B's actual callable
eligibility prerequisite. Its reached-payment consumer is designed to reuse the
existing empty-argument Requirements profile once that prerequisite is proved.
Nonempty selected-F conditional payment shapes remain explicitly unsupported:
the existing payment profile only admits empty inputs or its existing
`(function item,u8)` shape. No broader profile is inferred from tuple spreading.
Nonempty/open payment binders, const trait default/abstract member callbacks,
closures, imported callable eligibility and selected intrinsic F remain separate
required frontiers.

## Declared qualification

`pa31/tests/spec/const-eval-select-ctfe/contract.json` freezes expected outcomes
and source/client hashes before implementation or probes. It references the
unchanged original empty-ARG CTFE positive and six independently qualified
runtime semantic opposites, and declares twelve distinct new contexts:

- Owned non-Copy tuple transfer; mapped callback through checked generic returns;
  callback reference binders; evaluated operand order; and repeated empty-ARG
  calls. Their clients distinguish CTFE's F from runtime's G.
- An unreached selector with looping F, and a reached selector with looping G.
  Both must return seven without executing the loop or the selector placeholder.
- Conditional empty/nonempty callbacks retain the explicit prerequisites above.
- A selected constant bounds fault, its successful runtime G opposite, and
  reuse of a moved owned tuple argument are declared Rust hypotheses. New Rust
  results are not yet observed or claimed; disagreements must be retained.

Private controls are declared for wrong selector source/full instance, wrong
evaluated selector, wrong F declaration, changed requested/replayed F full
instance, wrong ARG tuple/arity, wrong opened selector signature/RET, foreign
Code/owner/public/internal signatures, wrong frame/block/position/Call, cold and
warm callback entry counts, fresh/warm Conditional denial, one-use payment,
changed caller Environment and selector-to-F payment substitution. Every
positive uses genuine source proof and actual materialized code. Mutations are
opponents against a retained checked witness, not fabricated positive authority.

No heavy C command is queued. Independent probes and candidate validation wait
for the integrator's release/performance boundary and normal resource admission.
