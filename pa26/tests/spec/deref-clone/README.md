# Deref and target Clone contracts (PA26)

This packet owns13 required inputs (10 accepts/3 rejects). Independent Rust outcomes are fixed. The source25 typing/24 prepared-root outcomes and2 actual source native programs are qualified; ordinary15 imports and the generic borrowed-reference artifact remain required. No course oracle or whole assignment is activated.

The actual lead is original core cell.rs1449, RefCell::new(self.borrow().clone()). Concrete/explicit controls distinguish receiver lookup from the later actual Deref call. Static associated clone has no receiver; an actual wrapper Clone and a borrowed-reference Clone retain priority. Missing Clone remains an error.

PA26 owns ordinary consumer contracts; PA30 owns the ordinary native C ABI observer; PA31 owns actual source language-item contracts. The shared observer requires `deref-clone 23 123`: borrow1, actual Deref2, selected target clone3 exactly once. Both independent Rust/C executions agree; candidate execution remains required.

Seventeen minimal source inputs reuse the previous borrow-coercions prefix exactly; three module-scoped cases retain those declarations under ops to test lexical visibility. Full-source cases reuse the union-clone foundation and exact original Deref/DerefMut trait declarations including const stability. Rust sources are pinned at88d9e12ae178fab0fb5cc050a94da85685d449ea; existing third_party attribution/checksums are unchanged. Four first full-header drafts lacked required const stability metadata; their bytes/errors remain in obj/parallel/refcell-clone/draft-header-review.json. Corrected inputs keep all four required accept outcomes. No Rust-valid unsupported positive was inverted.

See manifest.json and independent.json for every input, auxiliary hash and observed baseline phase. Raw evidence is B obj/parallel/refcell-clone.

Same-member contrasts require two distinct applications of the Deref member at one syntax node. The second independent Rust/C observer requires `deref-member 23 12`. Member identity alone cannot transfer caller-only restrictions to implicit protocol calls. The generic borrowed-reference return retains successful inference and a REQUIRED complete RootBuilder artifact at the generic aggregate instantiated-field gate; see the PA31 pending.json. It has not become a language rejection.

The result-context trait control requires a valid `Clone::clone(&guard)` coercion: the unknown result Self is shared with the direct formal input. A result-only Self may not supply an unrelated function-pointer argument prefix. The existing maintainer argument_expectations negative remains unchanged.

The six accepted width8/32/128+noise0/128 inputs exercise actual early result-to-input hints and normal protocol proof. Inference6492/23580/91932 and trait selections55/199/775 have identical unrelated-noise counters; method steps43/probes1 stay fixed. See [qualified checkpoint](../../../../docs/pa26-deref-clone.md) for receipts, profiles and retained artifacts.
