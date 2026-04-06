# Verified Arithmetic Circuit Simplifier

A formally verified arithmetic expression simplifier written in **Lean 4**,
callable from **Rust** via FFI. The Lean side defines the AST, simplification
rules, and a machine-checked proof that simplification preserves semantics.
The Rust side provides a safe API on top of the Lean static library.

## Project layout

```
lean/                        Lean 4 project (lake)
  ArithCircuit.lean          root import
  ArithCircuit/
    ArithCircuit.lean        Expr AST, eval, simplify, correctness proof
    FFI.lean                 @[export] wrappers for C-level symbols
  lakefile.toml              build config
  lean-toolchain             pins the Lean 4 version

arith-circuit-rs/            Rust crate (uses lean-sys for runtime bindings)
  shim/closure_shim.c        thin C shim for lean_alloc_closure (4.23→4.29 compat)
  build.rs                   links Lean static libs
  src/
    ffi.rs                   extern "C" declarations for our @[export] functions
    lib.rs                   safe Rust Expr API + tests
  examples/
    demo.rs                  usage demo
    bench.rs                 Lean FFI vs native Rust performance comparison
```

## Key guarantee

`Expr.simplify_correct` is a Lean theorem stating that for every expression
`e` and environment `env`:

```
e.simplify.eval env = e.eval env
```

This is checked by Lean's kernel at compile time -- the simplifier can never
change the meaning of an expression.

## Quick start

Prerequisites: [`elan`](https://github.com/leanprover/elan) (Lean toolchain
manager), `rustc`/`cargo`, a C compiler.

```bash
# Build the Lean static library
cd lean && lake build ArithCircuit:static && cd ..

# Build + test the Rust crate
cd arith-circuit-rs && cargo test -- --test-threads=1

# Run the demo
cargo run --example demo
```

## Example

```rust
arith_circuit::init();

let e = Expr::mul(
    &Expr::add(&Expr::var(0), &Expr::constant(0)),
    &Expr::mul(&Expr::constant(1), &Expr::var(1)),
);

let s = e.simplify();       // (x0 * x1)
s.eval(&[3, 5])             // 15
s.eval_with(|i| [3, 5][i])  // 15, via closure
```
