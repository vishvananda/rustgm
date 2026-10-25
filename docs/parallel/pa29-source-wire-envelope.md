# Source2024Functions artifact contract

This is the isolated, unqualified LIB-007 activation candidate. Public source
transport has not yet passed its provider/artifact/fresh-consumer/native gate.
The existing legacy tags 0–9 and generic row versions 0/1 retain their bytes.

The additive standalone envelope is little-endian: magic `RGMETA\r\n`, schema
`u16=1`, target `u8=0` (Linux x86-64 GNU), language `u8=0` (pinned Rust1.98 /
Rust2024), profile `u8=10`, source row version `u16=0`, explicit defining edition
`u8=0` (Rust2024), actual crate identity and canonical dependency identities.
The fixed ordinary-source columns then encode the complete declaration, body,
slot, scope, binder, type, region, argument, clause, application, trait, impl,
function, formal, member, role, original and raw-input tables. There are no
invented empty legacy function, nominal, storage or public-name tables. EOF is
mandatory. Unknown versions/editions/forms and incomplete original coverage fail.

Bounds remain 16,000,000 bytes for the artifact and aggregate retained/scratch
storage, 1,000,000 charged components, 100,000 declaration/header/body rows,
1,024 dependencies and 8,000,000 name bytes. Encoding additionally reserves a
conservative output-capacity envelope beside the already retained interface;
decoding reserves all owned column capacities before allocating them, then runs
one complete header/domain/original-input validation. These supported limits do
not establish that an original-core artifact will fit; that requires measurement
when its complete semantic profile becomes supported.

The provider publisher consumes actual Checked generic originals and actual
ordinary Proved bodies with their same paired Contract. It transfers its whole
completed source stage directly to payload validation once, after ordinary group
proof and final assembly; inspection remains a separate observational API.
Import reserves authentic resolver declarations, installs canonical headers and
actual receiver impl obligations, reconstructs unpaid bodies/inputs and proves
all originals, including unused ones, before retaining an imported set. Every
reached instance then requires ordinary substitution and proof separately.

The first supported semantic profile is ordinary free Rust function move bodies
with genuine sizing/ordinary Clone/Copy/receiver headers and bodyless required
members/receiver impls. It has no fixture/name/arity/body-count special cases.
Const defaults, projections/borrow/call bodies, wider roles, stability contracts,
and macro provenance are explicitly unsupported. Until complete name-context
transport exists, any retained contextual item or non-root token mark causes
whole-provider rejection, including opaque members below root-captured owners.
The full genuine Clone default family and its defining cleanup domain remain
required successors; this candidate does not close all PA29 requirements.
