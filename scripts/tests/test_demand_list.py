"""The demand list's symbol parser and IR classifiers agree with known rustc output."""
import importlib.util
from pathlib import Path
import sys
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


v0 = load("rust_v0_symbols")
with mock.patch.dict(sys.modules, {"rust_v0_symbols": v0}):
    demand = load("demand_list")

# Symbols from a rustc 1.98.0 build; the demangled column is binutils c++filt output.
SYMBOLS = [
    ("_RNvNtCs4sB1VOlFq4v_3std2rt4init", "std::rt::init", "std::rt::init", "std", "std::rt", "fn"),
    ("_RNCNvNtCs4sB1VOlFq4v_3std5alloc8rust_oom0B5_", "std::alloc::rust_oom::{closure#0}",
     "std::alloc::rust_oom::{closure#0}", "std", "std::alloc", "closure"),
    ("_RINvNtCs8LGlZXjwXB4_4core3ptr9drop_glueINtNtCs11wCyon1wOk_5alloc3vec3VechEEBG_",
     "core::ptr::drop_glue::<alloc::vec::Vec<u8>>", "core::ptr::drop_glue", "core", "core::ptr",
     "drop-glue"),
    ("_RINvMNtCs8LGlZXjwXB4_4core3stre3getINtNtNtB5_3ops5range5RangejEECs6KjptBz88KQ_9addr2line",
     "<str>::get::<core::ops::range::Range<usize>>", "core::str::{impl#0}::get", "core",
     "core::str", "fn"),
    ("_RINvMNtCs8LGlZXjwXB4_4core5sliceSh8as_arrayKj4_ECsdc4Pu338LJH_11miniz_oxide",
     "<[u8]>::as_array::<4>", "core::slice::{impl#0}::as_array", "core", "core::slice", "fn"),
]


class Symbols(unittest.TestCase):
    def test_paths_keep_the_defining_impl_and_drop_generic_arguments(self):
        for symbol, demangled, definition, crate, module, kind in SYMBOLS:
            with self.subTest(symbol=symbol):
                path, _, _ = v0.parse(symbol)
                self.assertEqual(v0.demangle(symbol), demangled)
                self.assertEqual(v0.defining_path(path), definition)
                self.assertEqual(v0.defining_crate(path), crate)
                self.assertEqual(v0.module_path(path), module)
                self.assertEqual(v0.classify(path), kind)

    def test_instantiating_crate_is_reported_separately(self):
        _, crate, _ = v0.parse(SYMBOLS[4][0])
        self.assertEqual(crate[:2], ("crate", "miniz_oxide"))
        self.assertIsNone(v0.parse(SYMBOLS[0][0])[1])

    def test_non_rust_and_truncated_symbols_are_rejected(self):
        for symbol in ("main", "_ZN3foo3barE", "_RNvNtCs4sB1VOlFq4v_3std2rt4in"):
            with self.subTest(symbol=symbol), self.assertRaises(v0.Invalid):
                v0.parse(symbol)


class Instructions(unittest.TestCase):
    def features(self, line):
        found = set()
        demand.instruction_features(line, found)
        return found

    def test_atomics_carry_width_and_ordering(self):
        self.assertEqual(self.features("  %1 = load atomic i64, ptr %p acquire, align 8"),
                         {("atomic", "load i64 Acquire")})
        self.assertEqual(self.features("  store atomic i8 %v, ptr %p seq_cst, align 1"),
                         {("atomic", "store i8 SeqCst")})
        self.assertEqual(self.features("  %2 = atomicrmw add ptr %p, i32 %v monotonic, align 4"),
                         {("atomic", "rmw-add i32 Relaxed")})
        self.assertEqual(
            self.features("  %3 = cmpxchg weak ptr %p, i64 %a, i64 %b acq_rel acquire, align 8"),
            {("atomic", "cmpxchg-weak i64 AcqRel/Acquire")})
        self.assertEqual(self.features("  fence release"), {("atomic", "fence Release")})

    def test_vector_wide_integer_and_float_operations_are_classified(self):
        self.assertEqual(self.features("  %m = icmp eq <16 x i8> %a, %b"),
                         {("vector", "icmp <16 x i8>")})
        self.assertEqual(self.features("  %q = udiv i128 %a, %b"), {("i128", "udiv")})
        self.assertEqual(self.features("  %f = uitofp i128 %a to double"),
                         {("float", "uitofp i128->double"), ("i128", "uitofp")})

    def test_comments_and_switch_cases_are_not_operations(self):
        self.assertEqual(self.features("; call <i128 as core::fmt::Display>::fmt"), set())
        self.assertEqual(self.features("    i128 5, label %bb3"), set())

    def test_variadic_calls_name_the_callee(self):
        self.assertEqual(
            self.features("  %r = call i32 (ptr, i32, ...) @open64(ptr %p, i32 %f, i32 438)"),
            {("variadic-call", "open64")})


class Intrinsics(unittest.TestCase):
    def test_trimmed_and_qualified_callee_paths_are_recognised(self):
        declared = {"ctpop": False, "transmute": False, "simd::simd_eq": False}
        bare = {"ctpop": "ctpop", "simd_eq": "simd::simd_eq"}
        code = ("_1 = ctpop::<u64>(copy _2) -> [return: bb1, unwind unreachable]; "
                "_3 = intrinsics::transmute::<u8, i8>(copy _4) -> bb2; "
                "_5 = std::intrinsics::simd::simd_eq::<A, B>(copy _6, copy _7) -> bb3; "
                "_8 = helper::<u8>(copy _9) -> bb4;")
        self.assertEqual(demand.called_intrinsics(code, declared, bare),
                         ["ctpop", "transmute", "simd::simd_eq"])


if __name__ == "__main__":
    unittest.main()
