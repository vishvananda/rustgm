//! Append-only source vocabulary, independently checked against the pinned Rust tables.
use super::{DeclarationProfile, DeclarationTarget, GenericRequirement};

macro_rules! language_items {
    (
        existing { $($(#[$meta:meta])* $old:ident = $index:literal => [$($name:literal),*];)* }
        declarations { $($new:ident = $new_index:literal => $spelling:literal, $target:ident, $generics:expr;)* }
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum LangItem {
            $($(#[$meta])* $old = $index,)*
            $($new = $new_index,)*
        }
        impl LangItem {
            pub const ALL: [Self; language_items!(@count $($old)* $($new)*)] = [
                $(Self::$old,)* $(Self::$new,)*
            ];
            /// New declaration profiles still require their actual semantic consumers.
            pub(crate) const DECLARATIONS: [Self; language_items!(@count $($new)*)] = [$(Self::$new,)*];
            #[must_use]
            pub const fn index(self) -> usize { self as usize }
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($($name => Some(Self::$old),)*)*
                    $($spelling => Some(Self::$new),)*
                    _ => None,
                }
            }
            pub(crate) const fn declaration_profile(self) -> Option<DeclarationProfile> {
                match self {
                    $(Self::$new => Some(DeclarationProfile {
                        target: DeclarationTarget::$target,
                        generics: $generics,
                    }),)*
                    _ => None,
                }
            }
        }
    };
    (@count $($item:ident)*) => { <[()]>::len(&[$(language_items!(@unit $item)),*]) };
    (@unit $item:ident) => { () };
}

// Existing discriminants and spellings are frozen. Send is diagnostic-owned.
// The appended rows follow rustc_hir/lang_items.rs at 88d9e12ae178fab0fb5cc050a94da85685d449ea;
// weak-only foreign targets additionally follow rustc_hir/weak_lang_items.rs.
language_items! {
    existing {
        Sized = 0 => ["sized"];
        MetaSized = 1 => ["meta_sized"];
        PointeeSized = 2 => ["pointee_sized"];
        MemSizeConst = 3 => ["mem_size_const"];
        MemAlignConst = 4 => ["mem_align_const"];
        Copy = 5 => ["copy"];
        Clone = 6 => ["clone"];
        CloneFn = 7 => ["clone_fn"];
        LegacyReceiver = 8 => ["legacy_receiver"];
        Tuple = 9 => ["tuple_trait"];
        FnOnce = 10 => ["fn_once"];
        FnMut = 11 => ["fn_mut"];
        Fn = 12 => ["fn"];
        FnOnceOutput = 13 => ["fn_once_output"];
        Drop = 14 => ["drop"];
        Destruct = 15 => ["destruct"];
        Freeze = 16 => ["freeze"];
        DropGlue = 17 => ["drop_glue"];
        Unsize = 18 => ["unsize"];
        CoerceUnsized = 19 => ["coerce_unsized"];
        FormatArguments = 20 => ["format_arguments"];
        FormatArgument = 21 => ["format_argument"];
        Not = 22 => ["not"];
        Panic = 23 => ["panic"];
        PanicFmt = 24 => ["panic_fmt"];
        PanicImpl = 25 => ["panic_impl"];
        PanicDisplay = 26 => ["panic_display"];
        ConstPanicFmt = 27 => ["const_panic_fmt"];
        PointeeTrait = 28 => ["pointee_trait"];
        MetadataType = 29 => ["metadata_type"];
        DynMetadata = 30 => ["dyn_metadata"];
        PhantomData = 31 => ["phantom_data"];
        PanicLocation = 32 => ["panic_location"];
        PanicInfo = 33 => ["panic_info"];
        TrivialClone = 34 => ["trivial_clone"];
        TryTrait = 35 => ["Try"];
        TryFromResidual = 36 => ["from_residual"];
        TryFromOutput = 37 => ["from_output"];
        TryBranch = 38 => ["branch"];
        IntoTryType = 39 => ["into_try_type"];
        ControlFlowContinue = 40 => ["Continue"];
        ControlFlowBreak = 41 => ["Break"];
        PartialEq = 42 => ["eq"];
        StructuralPartialEq = 43 => ["structural_peq"];
        Ordering = 44 => ["Ordering"];
        PartialOrd = 45 => ["partial_ord"];
        DiscriminantKind = 46 => ["discriminant_kind"];
        DiscriminantType = 47 => ["discriminant_type"];
        Neg = 48 => ["neg"];
        /// Internal diagnostic-owned role, with no `lang` spelling or wire tag.
        Send = 49 => [];
        Sync = 50 => ["sync"];
        Unpin = 51 => ["unpin"];
        DispatchFromDyn = 52 => ["dispatch_from_dyn"];
        Shl = 53 => ["shl"];
        BitOr = 54 => ["bitor"];
        ResultOk = 55 => ["Ok"];
        ResultErr = 56 => ["Err"];
        Option = 57 => ["Option"];
        OptionSome = 58 => ["Some"];
        OptionNone = 59 => ["None"];
        SliceLen = 60 => ["slice_len_fn"];
        AllocLayout = 61 => ["alloc_layout"];
        Add = 62 => ["add"];
        Sub = 63 => ["sub"];
        OffsetOf = 64 => ["offset_of"];
        ConstParamTy = 65 => ["const_param_ty"];
        UnsafeUnpin = 66 => ["unsafe_unpin"];
        RangeMin = 67 => ["RangeMin"];
        RangeMax = 68 => ["RangeMax"];
        RangeSub = 69 => ["RangeSub"];
        Div = 70 => ["div"];
        From = 71 => ["From"];
        Iterator = 72 => ["iterator"];
        IteratorNext = 73 => ["next"];
        IntoIter = 74 => ["into_iter"];
        ContractBuildCheckEnsures = 75 => ["contract_build_check_ensures"];
        ContractCheckRequires = 76 => ["contract_check_requires"];
        ContractCheckEnsures = 77 => ["contract_check_ensures"];
        ManuallyDrop = 78 => ["manually_drop"];
        MaybeDangling = 79 => ["maybe_dangling"];
        BikeshedGuaranteedNoDrop = 80 => ["bikeshed_guaranteed_no_drop"];
        MaybeUninit = 81 => ["maybe_uninit"];
        PanicNounwind = 82 => ["panic_nounwind"];
        TransmuteTrait = 83 => ["transmute_trait"];
        TransmuteOpts = 84 => ["transmute_opts"];
        TypeInfo = 85 => ["type_info"];
        TypeId = 86 => ["type_id"];
    }
    declarations {
        UseCloned = 87 => "use_cloned", Trait, GenericRequirement::Any;
        FnPtrTrait = 88 => "fn_ptr_trait", Trait, GenericRequirement::Exact(0);
        FnPtrAddr = 89 => "fn_ptr_addr", RequiredMethod, GenericRequirement::Any;
        AsyncDrop = 90 => "async_drop", Trait, GenericRequirement::Any;
        AsyncDropInPlace = 91 => "async_drop_in_place", Function, GenericRequirement::Exact(1);
        Mul = 92 => "mul", Trait, GenericRequirement::Exact(1);
        Rem = 93 => "rem", Trait, GenericRequirement::Exact(1);
        BitXor = 94 => "bitxor", Trait, GenericRequirement::Exact(1);
        BitAnd = 95 => "bitand", Trait, GenericRequirement::Exact(1);
        Shr = 96 => "shr", Trait, GenericRequirement::Exact(1);
        AddAssign = 97 => "add_assign", Trait, GenericRequirement::Exact(1);
        SubAssign = 98 => "sub_assign", Trait, GenericRequirement::Exact(1);
        MulAssign = 99 => "mul_assign", Trait, GenericRequirement::Exact(1);
        DivAssign = 100 => "div_assign", Trait, GenericRequirement::Exact(1);
        RemAssign = 101 => "rem_assign", Trait, GenericRequirement::Exact(1);
        BitXorAssign = 102 => "bitxor_assign", Trait, GenericRequirement::Exact(1);
        BitAndAssign = 103 => "bitand_assign", Trait, GenericRequirement::Exact(1);
        BitOrAssign = 104 => "bitor_assign", Trait, GenericRequirement::Exact(1);
        ShlAssign = 105 => "shl_assign", Trait, GenericRequirement::Exact(1);
        ShrAssign = 106 => "shr_assign", Trait, GenericRequirement::Exact(1);
        Index = 107 => "index", Trait, GenericRequirement::Exact(1);
        IndexMut = 108 => "index_mut", Trait, GenericRequirement::Exact(1);
        UnsafeCell = 109 => "unsafe_cell", Struct, GenericRequirement::Any;
        UnsafePinned = 110 => "unsafe_pinned", Struct, GenericRequirement::Any;
        VaArgSafe = 111 => "va_arg_safe", Trait, GenericRequirement::Any;
        VaList = 112 => "va_list", Struct, GenericRequirement::Any;
        Deref = 113 => "deref", Trait, GenericRequirement::Exact(0);
        DerefMut = 114 => "deref_mut", Trait, GenericRequirement::Exact(0);
        DerefPure = 115 => "deref_pure", Trait, GenericRequirement::Exact(0);
        DerefTarget = 116 => "deref_target", AssociatedType, GenericRequirement::Any;
        Receiver = 117 => "receiver", Trait, GenericRequirement::Any;
        ReceiverTarget = 118 => "receiver_target", AssociatedType, GenericRequirement::Any;
        AsyncFn = 119 => "async_fn", Trait, GenericRequirement::Exact(1);
        AsyncFnMut = 120 => "async_fn_mut", Trait, GenericRequirement::Exact(1);
        AsyncFnOnce = 121 => "async_fn_once", Trait, GenericRequirement::Exact(1);
        AsyncFnOnceOutput = 122 => "async_fn_once_output", AssociatedType, GenericRequirement::Exact(1);
        CallOnceFuture = 123 => "call_once_future", AssociatedType, GenericRequirement::Exact(1);
        CallRefFuture = 124 => "call_ref_future", AssociatedType, GenericRequirement::Exact(2);
        AsyncFnKindHelper = 125 => "async_fn_kind_helper", Trait, GenericRequirement::Exact(1);
        AsyncFnKindUpvars = 126 => "async_fn_kind_upvars", AssociatedType, GenericRequirement::Exact(5);
        FusedIterator = 127 => "fused_iterator", Trait, GenericRequirement::Exact(0);
        Future = 128 => "future_trait", Trait, GenericRequirement::Exact(0);
        FutureOutput = 129 => "future_output", AssociatedType, GenericRequirement::Exact(0);
        AsyncIterator = 130 => "async_iterator", Trait, GenericRequirement::Exact(0);
        CoroutineState = 131 => "coroutine_state", Enum, GenericRequirement::Any;
        Coroutine = 132 => "coroutine", Trait, GenericRequirement::Exact(1);
        CoroutineReturn = 133 => "coroutine_return", AssociatedType, GenericRequirement::Exact(1);
        CoroutineYield = 134 => "coroutine_yield", AssociatedType, GenericRequirement::Exact(1);
        CoroutineResume = 135 => "coroutine_resume", RequiredMethod, GenericRequirement::Any;
        Pin = 136 => "pin", Struct, GenericRequirement::Any;
        CVoid = 137 => "c_void", Enum, GenericRequirement::Any;
        PanicBoundsCheck = 138 => "panic_bounds_check", Function, GenericRequirement::Exact(0);
        PanicMisalignedPointerDereference = 139 => "panic_misaligned_pointer_dereference", Function, GenericRequirement::Exact(0);
        PanicCannotUnwind = 140 => "panic_cannot_unwind", Function, GenericRequirement::Exact(0);
        PanicInCleanup = 141 => "panic_in_cleanup", Function, GenericRequirement::Exact(0);
        PanicNullPointerDereference = 142 => "panic_null_pointer_dereference", Function, GenericRequirement::Any;
        PanicInvalidEnumConstruction = 143 => "panic_invalid_enum_construction", Function, GenericRequirement::Any;
        CompilerMove = 144 => "compiler_move", Function, GenericRequirement::Exact(2);
        CompilerCopy = 145 => "compiler_copy", Function, GenericRequirement::Exact(2);
        TryTraitFromYeet = 146 => "from_yeet", Function, GenericRequirement::Any;
        CoercePointeeValidated = 147 => "coerce_pointee_validated", Trait, GenericRequirement::Exact(0);
        Poll = 148 => "Poll", Enum, GenericRequirement::Any;
        PollReady = 149 => "Ready", Variant, GenericRequirement::Any;
        PollPending = 150 => "Pending", Variant, GenericRequirement::Any;
        AsyncGenReady = 151 => "AsyncGenReady", InherentMethod, GenericRequirement::Exact(1);
        AsyncGenPending = 152 => "AsyncGenPending", AssociatedConst, GenericRequirement::Exact(1);
        AsyncGenFinished = 153 => "AsyncGenFinished", AssociatedConst, GenericRequirement::Exact(1);
        ResumeTy = 154 => "ResumeTy", Struct, GenericRequirement::Any;
        GetContext = 155 => "get_context", Function, GenericRequirement::Any;
        Context = 156 => "Context", Struct, GenericRequirement::Any;
        FuturePoll = 157 => "poll", RequiredMethod, GenericRequirement::Any;
        AsyncIteratorPollNext = 158 => "async_iterator_poll_next", RequiredMethod, GenericRequirement::Exact(0);
        IntoAsyncIterIntoIter = 159 => "into_async_iter_into_iter", RequiredMethod, GenericRequirement::Exact(0);
        IntoFutureIntoFuture = 160 => "into_future", RequiredMethod, GenericRequirement::Any;
        PinNewUnchecked = 161 => "new_unchecked", InherentMethod, GenericRequirement::Any;
        RangeFrom = 162 => "RangeFrom", Struct, GenericRequirement::Any;
        RangeFull = 163 => "RangeFull", Struct, GenericRequirement::Any;
        RangeInclusiveStruct = 164 => "RangeInclusive", Struct, GenericRequirement::Any;
        RangeInclusiveNew = 165 => "range_inclusive_new", InherentMethod, GenericRequirement::Any;
        Range = 166 => "Range", Struct, GenericRequirement::Any;
        RangeToInclusive = 167 => "RangeToInclusive", Struct, GenericRequirement::Any;
        RangeTo = 168 => "RangeTo", Struct, GenericRequirement::Any;
        RangeFromCopy = 169 => "RangeFromCopy", Struct, GenericRequirement::Any;
        RangeCopy = 170 => "RangeCopy", Struct, GenericRequirement::Any;
        RangeInclusiveCopy = 171 => "RangeInclusiveCopy", Struct, GenericRequirement::Any;
        RangeToInclusiveCopy = 172 => "RangeToInclusiveCopy", Struct, GenericRequirement::Any;
        CStr = 173 => "CStr", Struct, GenericRequirement::Any;
        Reborrow = 174 => "reborrow", Trait, GenericRequirement::Exact(0);
        CoerceShared = 175 => "coerce_shared", Trait, GenericRequirement::Exact(1);
        FieldRepresentingType = 176 => "field_representing_type", Struct, GenericRequirement::Exact(3);
        Field = 177 => "field", Trait, GenericRequirement::Exact(0);
        FieldBase = 178 => "field_base", AssociatedType, GenericRequirement::Exact(0);
        FieldType = 179 => "field_type", AssociatedType, GenericRequirement::Exact(0);
        FieldOffset = 180 => "field_offset", AssociatedConst, GenericRequirement::Exact(0);
        MemCpy = 181 => "memcpy_fn", ForeignFunction, GenericRequirement::Any;
        MemMove = 182 => "memmove_fn", ForeignFunction, GenericRequirement::Any;
        MemSet = 183 => "memset_fn", ForeignFunction, GenericRequirement::Any;
        MemCmp = 184 => "memcmp_fn", ForeignFunction, GenericRequirement::Any;
        Bcmp = 185 => "bcmp_fn", ForeignFunction, GenericRequirement::Any;
        StrLen = 186 => "strlen_fn", ForeignFunction, GenericRequirement::Any;
        PanicAddOverflow = 187 => "panic_const_add_overflow", Function, GenericRequirement::Any;
        PanicSubOverflow = 188 => "panic_const_sub_overflow", Function, GenericRequirement::Any;
        PanicMulOverflow = 189 => "panic_const_mul_overflow", Function, GenericRequirement::Any;
        PanicDivOverflow = 190 => "panic_const_div_overflow", Function, GenericRequirement::Any;
        PanicRemOverflow = 191 => "panic_const_rem_overflow", Function, GenericRequirement::Any;
        PanicNegOverflow = 192 => "panic_const_neg_overflow", Function, GenericRequirement::Any;
        PanicShrOverflow = 193 => "panic_const_shr_overflow", Function, GenericRequirement::Any;
        PanicShlOverflow = 194 => "panic_const_shl_overflow", Function, GenericRequirement::Any;
        PanicDivZero = 195 => "panic_const_div_by_zero", Function, GenericRequirement::Any;
        PanicRemZero = 196 => "panic_const_rem_by_zero", Function, GenericRequirement::Any;
        PanicCoroutineResumed = 197 => "panic_const_coroutine_resumed", Function, GenericRequirement::Any;
        PanicAsyncFnResumed = 198 => "panic_const_async_fn_resumed", Function, GenericRequirement::Any;
        PanicAsyncGenFnResumed = 199 => "panic_const_async_gen_fn_resumed", Function, GenericRequirement::Any;
        PanicGenFnNone = 200 => "panic_const_gen_fn_none", Function, GenericRequirement::Any;
        PanicCoroutineResumedPanic = 201 => "panic_const_coroutine_resumed_panic", Function, GenericRequirement::Any;
        PanicAsyncFnResumedPanic = 202 => "panic_const_async_fn_resumed_panic", Function, GenericRequirement::Any;
        PanicAsyncGenFnResumedPanic = 203 => "panic_const_async_gen_fn_resumed_panic", Function, GenericRequirement::Any;
        PanicGenFnNonePanic = 204 => "panic_const_gen_fn_none_panic", Function, GenericRequirement::Any;
        PanicCoroutineResumedDrop = 205 => "panic_const_coroutine_resumed_drop", Function, GenericRequirement::Any;
        PanicAsyncFnResumedDrop = 206 => "panic_const_async_fn_resumed_drop", Function, GenericRequirement::Any;
        PanicAsyncGenFnResumedDrop = 207 => "panic_const_async_gen_fn_resumed_drop", Function, GenericRequirement::Any;
        PanicGenFnNoneDrop = 208 => "panic_const_gen_fn_none_drop", Function, GenericRequirement::Any;
    }
}
