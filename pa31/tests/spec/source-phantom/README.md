# Actual source PhantomData marker

This planned PA31 source fixture retains the pinned Rust 1.98 `marker.rs`
unit marker, its PointeeSized formal, unconditional Copy/Clone/Freeze source
implementations and Clone body. The retained sizing, Clone, Destruct, receiver
and drop declarations are actual source prerequisites. Compiler knowledge adds
logical variance/ownership without a physical T field or callable permission.

The first local profile is intended to prove source construction, const zero
size/alignment, no physical drop, an actual Clone call for a non-Copy argument,
and lifetime covariance. It does not publish a PhantomData wire role or finish
Location, caller tracking, PanicInfo, formatting or assertion execution.
Independent std-backed dropck and auto-trait reducers are retained separately;
complete upstream stability/auto-trait families remain required.
