# Resume source declarations and pointer helper

The actual ResumeTy struct and get_context function are ordinary source
declarations. Their tags do not supply a special layout, unsafe exemption,
Copy/Clone implementation, lifetime extension or generated async execution.
Complete real fields and callable headers using the existing source nominal
and defined-function adapters, then check all demanded bodies normally.

Thirteen predicted outcomes are independently checked against pinned Rust1.98
before candidate changes. They cover arbitrary valid unused struct/function
headers, duplicates/wrong targets and genuine body/unsafe failures. The native
companion executes the exact original get_context body on inputs23/47 with the
already qualified selected NonNull source methods and a clearly authored test
Context. It does not claim the standard Context implementation or actual
ResumeTy Debug/Copy/Clone and Send/Sync graph. The complete original future/mod.rs
and helper interval remain unchanged as references; full original Context/Waker,
derive, async consumer and authenticated metadata requirements remain open.

Referencev1/v2 preserve missing and mismatched const-stability setup failures in
the inherited reduced NonNull provider. Referencev3 restores actual Clone/Destruct
and intrinsic const-stability policy without changing predicted acceptance,
then passes all thirteen outcomes and native23/47. Original source bytes and
outcomes remain frozen during candidate validation.
