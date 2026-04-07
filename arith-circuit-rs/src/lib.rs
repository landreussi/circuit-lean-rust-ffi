mod ffi;

use std::ffi::CStr;
use std::sync::Once;

use ffi::*;
use lean_sys::*;

static LEAN_INIT: Once = Once::new();

/// Initialize the Lean runtime.  Must be called before any other function.
pub fn init() {
    LEAN_INIT.call_once(|| unsafe {
        lean_initialize_runtime_module();
        lean_initialize_thread();
        lean_set_panic_messages(false);
        let res = initialize_arith__circuit_ArithCircuit(1);
        lean_set_panic_messages(true);
        assert!(lean_io_result_is_ok(res), "Lean module init failed");
        lean_dec(res);
        lean_io_mark_end_initialization();
    });
}

// ── Expr: safe wrapper around opaque Lean object ─────────────────

/// An opaque handle to a Lean `Expr` value.
/// Reference-counted; cloning increments the Lean refcount.
pub struct Expr {
    ptr: *mut lean_object,
}

unsafe impl Send for Expr {}
unsafe impl Sync for Expr {}

impl Expr {
    /// Create a constant expression: `Expr.const n`
    pub fn constant(n: i64) -> Self {
        unsafe {
            let lean_int = lean_int64_to_int(n);
            Expr {
                ptr: arith_expr_const(lean_int),
            }
        }
    }

    /// Create a variable reference: `Expr.var i`
    pub fn var(i: u32) -> Self {
        unsafe {
            let lean_nat = lean_unsigned_to_nat(i);
            Expr {
                ptr: arith_expr_var(lean_nat),
            }
        }
    }

    /// Create an addition: `Expr.add a b`
    pub fn add(a: &Expr, b: &Expr) -> Self {
        unsafe {
            lean_inc(a.ptr);
            lean_inc(b.ptr);
            Expr {
                ptr: arith_expr_add(a.ptr, b.ptr),
            }
        }
    }

    /// Create a multiplication: `Expr.mul a b`
    pub fn mul(a: &Expr, b: &Expr) -> Self {
        unsafe {
            lean_inc(a.ptr);
            lean_inc(b.ptr);
            Expr {
                ptr: arith_expr_mul(a.ptr, b.ptr),
            }
        }
    }

    /// Simplify the expression (e+0, 0+e, e*1, 1*e, const folding).
    pub fn simplify(&self) -> Self {
        unsafe {
            lean_inc(self.ptr);
            Expr {
                ptr: arith_expr_simplify(self.ptr),
            }
        }
    }

    /// Evaluate with a slice of variable bindings.
    /// `env[i]` is the value of `var i`; out-of-bounds defaults to 0.
    pub fn eval(&self, env: &[i64]) -> i64 {
        unsafe {
            let mut arr = lean_mk_empty_array();
            for &v in env {
                arr = arith_array_push(arr, lean_int64_to_int(v));
            }
            lean_inc(self.ptr);
            let result = arith_expr_eval(arr, self.ptr);
            lean_obj_to_i64(result)
        }
    }

    /// Evaluate with a callback: `f(i)` returns the value of `var i`.
    pub fn eval_with(&self, f: impl Fn(usize) -> i64) -> i64 {
        let trait_obj: &dyn Fn(usize) -> i64 = &f;
        unsafe {
            let raw: [usize; 2] = std::mem::transmute(trait_obj as *const dyn Fn(usize) -> i64);
            EVAL_CB_RAW.set(raw);
            let closure = shim_alloc_closure(env_trampoline as *mut std::ffi::c_void, 1, 0);
            lean_inc(self.ptr);
            let result = arith_expr_eval_fn(closure, self.ptr);
            EVAL_CB_RAW.set([0; 2]);
            lean_obj_to_i64(result)
        }
    }

    /// Pretty-print the expression.
    pub fn display(&self) -> String {
        unsafe {
            lean_inc(self.ptr);
            let lean_str = arith_expr_to_string(self.ptr);
            let cstr = CStr::from_ptr(lean_string_cstr(lean_str) as *const i8);
            let s = cstr.to_string_lossy().into_owned();
            lean_dec(lean_str);
            s
        }
    }
}

impl Drop for Expr {
    fn drop(&mut self) {
        unsafe {
            if !lean_is_scalar(self.ptr) {
                lean_dec(self.ptr);
            }
        }
    }
}

impl Clone for Expr {
    fn clone(&self) -> Self {
        unsafe {
            if !lean_is_scalar(self.ptr) {
                lean_inc(self.ptr);
            }
        }
        Expr { ptr: self.ptr }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display())
    }
}

// ── eval_with internals ──────────────────────────────────────────

thread_local! {
    static EVAL_CB_RAW: std::cell::Cell<[usize; 2]> = const { std::cell::Cell::new([0; 2]) };
}

/// Trampoline called by Lean when it evaluates the env closure.
/// Receives a Lean Nat, calls back into Rust, returns a Lean Int.
unsafe extern "C" fn env_trampoline(nat_arg: lean_obj_arg) -> lean_obj_res {
    unsafe {
        let idx = lean_unbox(nat_arg);
        let raw = EVAL_CB_RAW.get();
        if raw == [0; 2] {
            return lean_int64_to_int(0);
        }
        let f: *const dyn Fn(usize) -> i64 = std::mem::transmute(raw);
        let val = (*f)(idx);
        lean_int64_to_int(val)
    }
}

// ── Int extraction helper ────────────────────────────────────────

unsafe fn lean_obj_to_i64(obj: *mut lean_object) -> i64 {
    unsafe {
        if lean_is_scalar(obj) {
            lean_scalar_to_int64(obj)
        } else {
            panic!("big Int values not supported in FFI (value doesn't fit in scalar)");
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_and_eval() {
        init();

        // (x0 + 0) * (1 * x1)
        let expr = Expr::mul(
            &Expr::add(&Expr::var(0), &Expr::constant(0)),
            &Expr::mul(&Expr::constant(1), &Expr::var(1)),
        );
        assert_eq!(expr.display(), "((x0 + 0) * (1 * x1))");

        let simplified = expr.simplify();
        assert_eq!(simplified.display(), "(x0 * x1)");

        // eval with x0=3, x1=5
        assert_eq!(expr.eval(&[3, 5]), 15);
        assert_eq!(simplified.eval(&[3, 5]), 15);
    }

    #[test]
    fn test_const_folding() {
        init();

        // (2 + 3) * x0
        let expr = Expr::mul(
            &Expr::add(&Expr::constant(2), &Expr::constant(3)),
            &Expr::var(0),
        );
        let simplified = expr.simplify();
        assert_eq!(simplified.display(), "(5 * x0)");
        assert_eq!(simplified.eval(&[7]), 35);
    }

    #[test]
    fn test_eval_with_closure() {
        init();

        // x0 * x1 + x2
        let expr = Expr::add(&Expr::mul(&Expr::var(0), &Expr::var(1)), &Expr::var(2));

        let result = expr.eval_with(|i| match i {
            0 => 4,
            1 => 5,
            2 => 6,
            _ => 0,
        });
        assert_eq!(result, 26); // 4*5 + 6
    }
}
