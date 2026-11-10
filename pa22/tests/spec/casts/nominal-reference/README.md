# Explicit nominal reference unsizing casts (PA22)

The original core as_request failure reduces to an ordinary explicit slice-tail
cast. Its implicit twin passes. Cast preparation and validation previously only
recognized direct array-to-slice references. These36 declared cases have24 real
native positives and12 semantic negatives, independently object/execution checked
before implementation. Each positive main exercises the conversion; there are no
empty-entrypoint runtime claims.

Prefix/payload reads, mutable writes, exact slice metadata, empty/ZST/non-Copy
storage, generic/projected/nested tails, promotion/CTFE, evaluation count and
reborrows use the existing typed conversion. Rejections preserve elements, prefix
arguments, owner identity, mutability, lifetimes, loans and packed alignment.
Width8/32/128 and128 unrelated types bound actual call work. Object metadata and
virtual dispatch counterparts live in PA23, with their own manifest entries.

Every source is registered in the owning assignment manifest. Expected outcomes
and independent native observations precede generated references. Upstream source
bytes and provenance remain unchanged. The complete original source-language
Unsize nominal-tail proof is separate PA31 work; this packet adds no permission
from pointer layout or spelling.
