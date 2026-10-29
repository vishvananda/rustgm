# Source always-const functions

PA31 retains the actual `rustc_comptime` declaration/effect; PA25 checks its defining body and source call context. Marked non-const syntax has an effective const body. Direct function-item calls, including aliases and inherent methods, require an actual constant context; an ordinary const-fn body is insufficient. Actual inline const bodies and marked bodies qualify.

Pinned Rust accepts legitimate function-pointer erasure and runtime execution of this checked body. The pointer case is a required positive, with independently observed native7; no blanket reification restriction or synthetic CTFE fold is allowed. Ordinary const bodies, regular/const API stability and unsafe permission remain independent.

Passing the item through an ordinary generic `F: Fn(u32) -> u32` caller also executes at runtime and returns7. A closure created in a constant retains its own runtime body and cannot directly call the marked item; an explicit inline const inside a runtime closure can call it. The separately observed ordinary `rustc_attrs` and genuine macro allowance cases retain their actual gate origin.

The fixtures reuse already qualified real NoCore sizing/receiver/Clone/Copy/Destruct declarations and actual primitive implementations. Their source functions are authored policy tests, not replacements for the original type-info intrinsics. The first failed Copy-less foundation and the rejected pointer hypothesis remain in raw obj evidence. Metadata transports remain explicitly unsupported for this marker, including otherwise const-capable profiles.

`contract.json` binds the unchanged original core `type_id_vtable` interval and its file hash. Its genuine intrinsic execution and original panic/body dependencies remain required separately. The initial generic identity fixture omitted the real `Freeze` declaration; the retained corrected observation adds that prerequisite and preserves the marked identity body.
