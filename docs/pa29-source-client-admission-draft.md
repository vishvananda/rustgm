# Source-client admission draft

This isolated source patch is unqualified and is not part of the current public
compiler checkpoint. It depends on Worker A completing the actual source payload
validator and its sealed `SourceDescriptor`, `SourceProfile`,
`Interface::source_descriptor` and `Dependencies::source_provider_index` APIs.
There is no successful client fixture or complete upstream library claim yet.

`Package` retains one optional boxed source selection. The provider variant owns
its existing local declarations; the client variant owns the distinct current
identity, exact provider identity, parsed feature requests, an Arc to the actual
validated dependency inventory and the selected artifact index. Ordinary packages
retain no source allocation and the existing 248-byte layout assertion.

Admission requires the exact current crate name/identity, Rust 2024, `no_core`
and its explicit language gate. A single canonical artifact search must find the
complete supported own-provider payload before bootstrap definitions are disabled.
Empty, legacy, wrong-identity and unsupported payloads cannot activate source mode.
Repeated inventory checks use the same immutable allocation and indexed actual
descriptor; matching names or an Arc alone do not authenticate a provider.

The provider-only accessor still governs local language and builtin-macro claims.
Both source modes suppress synthetic library fallback and share the language
feature checks. Parsed library requests remain separate. These checks admit a
configuration only; they grant no imported role, body, caller proof or CTFE
permission. Worker A owns completed catalog installation/remapping and report
binding. Its end-to-end source-provider/export/decode/client/native transaction
must qualify the successful path and changed-inventory opponents before release.

The draft adds exact rejection controls and preserves the ordinary empty-policy
and layout controls. They have not been compiled or run yet. Formatting has been
applied; no guard/build was started in this checkout. Combine the real metadata
facade before running audit, strict Clippy, the affected provider/client controls
and independent/native tests. Do not introduce a temporary accepting descriptor.
