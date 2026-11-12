# Let-condition captures (PA24)

A closure body's computed `if let` or `while let` scrutinee is consumed like a
computed match's, as pinned core's `read_radix_max_digits` loops over digits
inside a closure that copies its captures. The pattern's bindings are local to
the closure body.

- `while-let-digits` is core's shape: copied captures, a computed `while let`
  scrutinee and an early return;
- `if-let-copy` uses a copied capture in an `if let` condition, and
  `if-let-nested-closure` passes a nested closure sharing the capture to the
  scrutinee's call;
- `while-let-mutable` is an `FnMut` closure mutating its capture in the loop,
  and `while-let-owned` a `move` closure consuming its capture once after the
  loop;
- `while-let-moved` moves a capture in every iteration (E0382), and
  `if-let-fn-mutation` assigns a capture from an `Fn` closure (E0594); rustc
  rejects both.

Every positive executes and checks its result. Outcomes were declared before
rustc 1.98.0 and before any RustGM run of these bytes. See
[the checkpoint](../../../../docs/pa24-let-condition-captures.md).
