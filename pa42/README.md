# RustGM Programming Assignment 42 — Target features and assembly

The graded suite for this assignment is pending.

Implement the typed target-feature, vector layout, and assembly rules
for the course's Linux x86-64 target. Use the pinned CPPGM backend and
its verified capabilities. Complete the core prerequisite below before
[PA32](../pa32/README.md). The broader work can follow the bootstrap.

## Required core prerequisite

Use [PA31](../pa31/README.md)'s source binding and feature rules to
identify the assembly builtin. Reuse [PA27](../pa27/README.md)'s unsafe
place checks and [PA28](../pa28/README.md)'s typed memory and intrinsic
effects. The original `_mm_stream_ps` exercises the first end-to-end path.
Preserve its generated template, pointer register, real 128-bit vector
operand, `nostack`, and `preserves_flags` options.

1. Retain the parsed template, operands, modifiers, and source spans
   through expansion. Check register and operand types, option
   combinations, initialized inputs, writable outputs, unsafe permission,
   and actual target features.
2. Preserve left-to-right operand evaluation and left-to-right output
   writes after the assembly effect. Retain memory, flags, clobbers, and
   divergence in the typed CFG. Const and symbol operands keep their
   constant and relocation identities.
3. Implement the required SIMD storage, ABI, and lane checks for the
   actual substituted type. Keep the vector operand in its selected
   register class and enforce call compatibility.
4. Emit the required non-temporal store through the backend. Link the
   original function and check every input bit, the target storage,
   surrounding sentinels, and the emitted instruction. A source body
   exported to a consumer retains the checked operand and target facts.

## Broader target closure

Complete the target configurations, operand and modifier forms, explicit
register aliases, ABI clobbers, labels, symbols, global and naked assembly,
and unwind restrictions required by the selected profile. Check active
features and target-specific declarations against real target facts.
Architecture-specific feature requests and backend capability queries
do not change the configured target.

## Tool surface

Compile through the typed LowIR pipeline and link against the pinned
backend. Report an unavailable backend operation separately from an invalid
Rust assembly invocation.

## Check your implementation

Exercise side effects, aliased outputs, uninitialized inputs, divergence,
feature restrictions, constants, symbols, and ordinary code without
assembly. Observe executable results and instruction selection. Use an
independently compiled caller when checking the C ABI.
