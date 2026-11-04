# Default-body stability

PA31 owns the direct rustc_default_body_unstable declaration and feature identity.
PA26/PA29 own actual default/override selection and authenticated provided-body
transport; PA25 retains const-call rules and PA39 retains real destruction.
PA31 remains planned. Recognizing a marker is not an original library artifact.

Contract-v3 freezes 48 programs against Rust 1.98.0 revision
88d9e12ae178fab0fb5cc050a94da85685d449ea. Primary sources cover attribute parsing,
feature collection, stability queries and trait implementation checking. Five
fresh rustc clients use an actual provider rlib. An external impl inheriting the
provided body fails without the feature, even if unused; enabling the feature or
providing an explicit override succeeds. Calling an existing provider-side impl
through the stable API also succeeds. Local impls do not need the body feature.

The pinned parser shares the broad stability target list. This marker can belong
to a free function, struct, module, trait, const, type alias, field, variant, crate
or a type parameter with a default, including bodyless declarations. It creates
no missing body. It is neither inherited API instability nor const permission.
A required method still needs an implementation; an invalid provided body still
fails checking. Regular, const and default-body policies may coexist on one
actual owner and each axis has independent duplicate checks. Former names and
implied-by relationships retain the existing canonical feature rules.

Capture uses one (NodeKey, PolicyKind) index instead of separate regular/const
maps plus another body map. The existing policy vector owns the parsed default
policy and source origin. No ordinary AST/body/semantic fact field or speculative
body query is added. Local bodies pass the ordinary checker and real generic
provided/override lowering. The native observer prints 7 9. Old metadata profiles
explicitly reject policies they cannot preserve.

The candidate table binds 43 local/source-provider cases to contract-v3. The five
reference clients remain required RustGM imported body-policy consumer work.
The current provided-body metadata family has no availability contract; the
module availability family has no complete provided-body contract. Their future
composition must preserve old wire bytes, encode direct body policy with actual
owner/body identity and explicit absence, and check inherited use at external
impl creation. It must distinguish explicit overrides and stable API calls,
including unused impls, macro feature allowances and constant/destructor paths.
Dropping the marker or making every call to the stable method unstable is wrong.

The tests also check regular/const separation, field/variant/generic/crate targets,
failed/duplicate capture, existing const/rename/implication/variant controls,
metadata fences and work growth through 16/64/256 declarations. Full original
Drop/Pin behavior and original core/alloc/std artifacts remain requirements.

Contract-v1 and its diagnostics remain intact. Its staged public impls lacked
required stability annotations. setup-v2 adds the real impl annotation without
changing any behavior prediction; all39 observations pass. V3 adds nine target
and source-provider controls; all48 observations pass. The observer and reference
rlibs are only validation inputs, never a Rust frontend fallback. Commands,
hashes, diagnostics and predecessor failures remain under
obj/parallel/default-bodies-v1/oracle-v1 through oracle-v3.
