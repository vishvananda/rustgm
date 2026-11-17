# Source compound-assignment RHS coercions

The selected AddAssign implementation controls the RHS type. The exact mutable LHS place, two-phase receiver, and primitive compound assignment remain under the ordinary checked rules. The tests cover a source Deref before the call, a grouped place with a direct overloaded RHS, and a primitive control. Immutable storage, a closed incompatible RHS and overlapping receiver access must fail. The pinned source compiler accepted the three positives and rejected the three opponents for their declared causes.

The two native traces distinguish place evaluation, RHS evaluation, deref and assignment, and require each observed operation once. The same C observer is used through the existing RHS integration root. The packet isolates source behavior from the original String build and from the ordinary profile.
