// Minimal shim: lean_alloc_closure is static inline in lean.h and its
// implementation changed between Lean 4.23 and 4.29 (alloc_small_object →
// alloc_object). lean-sys targets 4.23, so we compile this against the real
// lean.h to get the correct version.
#include <lean/lean.h>

lean_object *shim_alloc_closure(void *fun, unsigned arity, unsigned num_fixed) {
    return lean_alloc_closure(fun, arity, num_fixed);
}
