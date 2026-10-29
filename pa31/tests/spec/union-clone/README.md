# Source union Clone

The frozen contract retains the unchanged core escape union and the original
`AssertParamIsCopy` declaration from pinned core source. The builtin recipe must
emit the real unsafe TrivialClone marker and ordinary Clone method, add Copy to
type-parameter bounds, assert Copy for the actual Self, and copy `*self` without
reading a field or calling a field Clone method. Lifetime/const/where syntax is
preserved; declaration defaults are omitted from impl formals.

The authored SourceCore runtime reuses the genuine existing foundation and adds
the exact normal Copy assertion helper. Its scalar and array union clones use
real Copy/Clone impls and observe initialized nonzero data. The array Copy/Clone
impl bodies are authored restricted `T:Copy` consumers, not replacements for the
full original library implementations. Copy-only, record/enum behavior, actual
helper/trait identity, missing Copy, typed replay and attempted payment remain
part of the focused controls. Ordinary Rust expansion is separate from source
provider/runtime proof; neither substitutes for original core artifacts.
