# Typed receiver reads through union storage

Terminal full42190605652 exposed a genuine false acceptance in the existing
reference-field-store and tuple-reference-field-store opponents. Rust still
rejects them: following the stored reference reads a union field, even when the
final operation is an assignment. Direct union field replacement remains legal.

The Field worklist previously checked the receiver before publishing its actual
autoderef count, so the existing unsafe check silently returned early. This repair
invokes the same union-place read kernel from the genuine typed reference read,
and from first custom-Deref receiver discovery while its lexical unsafe context
is active. Late chosen-method preparation supplies no unsafe permission. No
syntactic blanket union rejection, new authority table or limit is introduced.

The existing complete union matrix, both original CI failures, their unsafe-block
counterparts and ordinary/direct nested store positives all pass through pinned
Rust and the full CLI. Two additional actual source DerefMut fixtures freeze
Rust1/E0133 and Rust0 before candidate observation. Both candidate outcomes match,
including the exact unsafe-operation cause and normal lowering for the positive.

Qualified full-source receipt: obj/qualification/fast/1793957699542515371/receipt.json
(8 exact tests, strict/fmt/architecture pass,107.67s). Independent source pair:
obj/parallel/async-iterator-v1/union-field-oracle/receipt.json. The later paired
capture/native checkpoint adds no permission to this union repair. Root retains
ownership of full-CI private measurement repairs and successor qualification.
