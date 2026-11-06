# Source const cleanup follow-ons

These independently frozen authored consumers reuse the attributed source-const-drop foundation. They preserve its actual standard language declarations and required-member Drop header variant; they do not claim complete upstream Drop/Pin module closure. Original prior inputs and outcomes remain unchanged.

The independently frozen value-qualification controls are inventoried in
[histo.json](histo.json) (18 move, copy, overwrite, exposure and aggregate cases),
[qscope.json](qscope.json) (five actual field-borrow, conditional-bound and lifetime
cases), and [qcopy.json](qcopy.json) (four source-Copy field versus whole-Destruct
cases). Their exact source hashes and predicted Rust outcomes were recorded
before candidate observation. The original whole-source inputs remain intact.

`mfield.rs` and `rfield.rs` differ in the actual field borrowed before a whole
move: the possible nonconst field retains qualification, while the scalar field
permits clearing it. `cpfld.rs` requires actual scoped Copy proof for an unknown
field without reading it. `cpwhol.rs` still demands the stronger explicit const
Destruct bound. These controls do not authorize interpreter cleanup or weaken
physical initialization, reads, moves, loans, unsafe checks, or ordinary body
validation.
