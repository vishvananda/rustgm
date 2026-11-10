# Nominal object-tail reference casts (PA23)

These five declared counterparts extend PA22's nominal reference cast coverage
with object metadata: three independently executed positives and two independently
rejected proofs. They exercise real shared/mutable virtual dispatch, prefix/tail
storage and a local borrowed representation. Missing principal evidence and an
escaping object lifetime still reject. Every input is registered in PA23's
manifest and checked independently before generating text references. No backend
contract or original upstream source changes.
