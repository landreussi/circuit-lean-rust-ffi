// C shim wrapping Lean's static-inline runtime functions.
// Compiled by build.rs using the `cc` crate, linked into the Rust binary.

#include <lean/lean.h>
#include <stdint.h>

// ── Object lifecycle ──────────────────────────────────────────────
void shim_inc(lean_object *o)  { lean_inc(o); }
void shim_dec(lean_object *o)  { lean_dec(o); }

// ── Scalar boxing (tagged pointers) ──────────────────────────────
lean_object *shim_box(size_t n)         { return lean_box(n); }
size_t       shim_unbox(lean_object *o) { return lean_unbox(o); }
int          shim_is_scalar(lean_object *o) { return lean_is_scalar(o); }

// ── Int ↔ int64 ──────────────────────────────────────────────────
lean_object *shim_int64_to_int(int64_t n)       { return lean_int64_to_int(n); }
int64_t      shim_scalar_to_int64(lean_object *o){ return lean_scalar_to_int64(o); }
lean_object *shim_unsigned_to_nat(unsigned n)    { return lean_unsigned_to_nat(n); }

// ── Strings ──────────────────────────────────────────────────────
const char *shim_string_cstr(lean_object *o) { return lean_string_cstr(o); }
size_t      shim_string_len(lean_object *o)  { return lean_string_size(o) - 1; }

// ── IO world ─────────────────────────────────────────────────────
lean_object *shim_io_mk_world(void)                  { return lean_io_mk_world(); }
int          shim_io_result_is_ok(lean_object *r)     { return lean_io_result_is_ok(r); }

// ── Arrays ───────────────────────────────────────────────────────
lean_object *shim_mk_empty_array(void) { return lean_mk_empty_array(); }

// ── Closures (for passing Rust function pointers to Lean) ────────
lean_object *shim_alloc_closure(void *fun, unsigned arity, unsigned num_fixed) {
    return lean_alloc_closure(fun, arity, num_fixed);
}

// Thread-local callback: Rust sets this before calling eval_fn.
// The trampoline reads it when Lean calls back into C.
static __thread int64_t (*_env_cb)(size_t);

void shim_set_env_callback(int64_t (*cb)(size_t)) {
    _env_cb = cb;
}

// Called by Lean when it evaluates the env closure.
// Receives a Lean Nat, calls back into Rust, returns a Lean Int.
lean_object *shim_env_trampoline(lean_object *nat_arg) {
    size_t idx = lean_unbox(nat_arg);
    int64_t val = _env_cb(idx);
    return lean_int64_to_int(val);
}

// Build a Lean (Nat → Int) closure that delegates to the thread-local callback.
lean_object *shim_make_env_closure(void) {
    return lean_alloc_closure((void *)shim_env_trampoline, 1, 0);
}
