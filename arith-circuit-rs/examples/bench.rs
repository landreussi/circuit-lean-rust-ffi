use arith_circuit::{init, Expr as LeanExpr};
use std::rc::Rc;
use std::time::{Duration, Instant};

// ── Native Rust Expr (Rc-based, structure sharing like Lean) ────

#[derive(Clone)]
enum Expr {
    Const(i64),
    Var(u32),
    Add(Rc<Expr>, Rc<Expr>),
    Mul(Rc<Expr>, Rc<Expr>),
}

impl Expr {
    fn constant(n: i64) -> Rc<Self> { Rc::new(Expr::Const(n)) }
    fn var(i: u32) -> Rc<Self> { Rc::new(Expr::Var(i)) }
    fn add(a: Rc<Expr>, b: Rc<Expr>) -> Rc<Self> { Rc::new(Expr::Add(a, b)) }
    fn mul(a: Rc<Expr>, b: Rc<Expr>) -> Rc<Self> { Rc::new(Expr::Mul(a, b)) }

    fn eval(&self, env: &[i64]) -> i64 {
        match self {
            Expr::Const(n) => *n,
            Expr::Var(i) => env.get(*i as usize).copied().unwrap_or(0),
            Expr::Add(a, b) => a.eval(env).wrapping_add(b.eval(env)),
            Expr::Mul(a, b) => a.eval(env).wrapping_mul(b.eval(env)),
        }
    }
}

/// Simplify, returning the *same* Rc when nothing changed (structure sharing).
fn simplify(e: &Rc<Expr>) -> Rc<Expr> {
    match e.as_ref() {
        Expr::Const(_) | Expr::Var(_) => Rc::clone(e),
        Expr::Add(a, b) => {
            let a2 = simplify(a);
            let b2 = simplify(b);
            match (a2.as_ref(), b2.as_ref()) {
                (Expr::Const(0), _) => b2,
                (_, Expr::Const(0)) => a2,
                (Expr::Const(m), Expr::Const(n)) => Expr::constant(m + n),
                _ => {
                    // Reuse the original node if children didn't change
                    if Rc::ptr_eq(&a2, a) && Rc::ptr_eq(&b2, b) {
                        Rc::clone(e)
                    } else {
                        Expr::add(a2, b2)
                    }
                }
            }
        }
        Expr::Mul(a, b) => {
            let a2 = simplify(a);
            let b2 = simplify(b);
            match (a2.as_ref(), b2.as_ref()) {
                (Expr::Const(1), _) => b2,
                (_, Expr::Const(1)) => a2,
                (Expr::Const(m), Expr::Const(n)) => Expr::constant(m * n),
                _ => {
                    if Rc::ptr_eq(&a2, a) && Rc::ptr_eq(&b2, b) {
                        Rc::clone(e)
                    } else {
                        Expr::mul(a2, b2)
                    }
                }
            }
        }
    }
}

fn to_lean(e: &Expr) -> LeanExpr {
    match e {
        Expr::Const(n) => LeanExpr::constant(*n),
        Expr::Var(i) => LeanExpr::var(*i),
        Expr::Add(a, b) => LeanExpr::add(&to_lean(a), &to_lean(b)),
        Expr::Mul(a, b) => LeanExpr::mul(&to_lean(a), &to_lean(b)),
    }
}

fn node_count(e: &Expr) -> usize {
    match e {
        Expr::Const(_) | Expr::Var(_) => 1,
        Expr::Add(a, b) | Expr::Mul(a, b) => 1 + node_count(a) + node_count(b),
    }
}

// ── Expression generators ───────────────────────────────────────

fn make_small_expr(seed: u64, num_vars: u32) -> Rc<Expr> {
    let mut s = seed;
    let mut next = || -> u64 {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };

    let leaf = |s: &mut dyn FnMut() -> u64| -> Rc<Expr> {
        let v = s();
        match v % 5 {
            0 => Expr::constant(0),
            1 => Expr::constant(1),
            2 => Expr::constant((v % 20) as i64),
            _ => Expr::var((v % num_vars as u64) as u32),
        }
    };

    let a = leaf(&mut next);
    let b = leaf(&mut next);
    let c = leaf(&mut next);
    let d = leaf(&mut next);
    let e = leaf(&mut next);
    let f = leaf(&mut next);

    let ab = if next() % 2 == 0 { Expr::add(a, b) } else { Expr::mul(a, b) };
    let cd = if next() % 2 == 0 { Expr::add(c, d) } else { Expr::mul(c, d) };
    let ef = if next() % 2 == 0 { Expr::add(e, f) } else { Expr::mul(e, f) };
    let abcd = if next() % 2 == 0 { Expr::add(ab, cd) } else { Expr::mul(ab, cd) };
    if next() % 2 == 0 { Expr::add(abcd, ef) } else { Expr::mul(abcd, ef) }
}

/// Addition-only huge tree. Constants are 0 or 1 for simplification opportunities.
fn make_huge_expr(depth: u32, num_vars: u32) -> Rc<Expr> {
    fn next(seed: &mut u64) -> u64 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        *seed
    }
    fn build(depth: u32, num_vars: u32, seed: &mut u64) -> Rc<Expr> {
        if depth == 0 {
            let v = next(seed);
            return match v % 5 {
                0 => Expr::constant(0),
                1 => Expr::constant(1),
                _ => Expr::var((v % num_vars as u64) as u32),
            };
        }
        let left = build(depth - 1, num_vars, seed);
        let right = build(depth - 1, num_vars, seed);
        Expr::add(left, right)
    }
    let mut seed = 0xDEAD_BEEF_u64;
    build(depth, num_vars, &mut seed)
}

// ── Timing ──────────────────────────────────────────────────────

fn bench<F: FnMut()>(label: &str, iters: u32, mut f: F) -> Duration {
    for _ in 0..3 { f(); }
    let start = Instant::now();
    for _ in 0..iters { f(); }
    let elapsed = start.elapsed();
    let per_iter = elapsed / iters;
    println!("  {label:42} {iters:>6} iters  total {elapsed:>12.3?}  per-iter {per_iter:>10.3?}");
    elapsed
}

fn main() {
    init();

    let num_vars: u32 = 8;
    let env: Vec<i64> = (0..num_vars as i64).map(|i| i * 3 + 1).collect();

    // ════════════════════════════════════════════════════════════════
    println!("═══ Experiment 1: many small expressions (50M calls) ═══\n");
    // ════════════════════════════════════════════════════════════════

    let n_small = 50_000_usize;
    let small_exprs: Vec<Rc<Expr>> =
        (0..n_small as u64).map(|i| make_small_expr(i + 1, num_vars)).collect();
    let lean_small: Vec<LeanExpr> =
        small_exprs.iter().map(|e| to_lean(e)).collect();

    // 1000 passes over the 50k expressions = 50M simplify calls
    let iters = 1000_u32;

    // -- Lean simplify
    bench("Lean simplify", iters, || {
        for e in &lean_small { std::hint::black_box(e.simplify()); }
    });

    // -- Rust simplify
    bench("Rust simplify (Rc, sharing)", iters, || {
        for e in &small_exprs { std::hint::black_box(simplify(e)); }
    });

    // Pre-simplify for eval benchmarks
    let lean_simplified: Vec<LeanExpr> = lean_small.iter().map(|e| e.simplify()).collect();
    let rust_simplified: Vec<Rc<Expr>> = small_exprs.iter().map(|e| simplify(e)).collect();

    // -- Lean eval (array)
    bench("Lean eval (array)", iters, || {
        for e in &lean_simplified { std::hint::black_box(e.eval(&env)); }
    });

    // -- Lean eval_with (closure)
    bench("Lean eval_with (closure)", iters, || {
        for e in &lean_simplified { std::hint::black_box(e.eval_with(|i| env[i])); }
    });

    // -- Rust eval
    bench("Rust eval", iters, || {
        for e in &rust_simplified { std::hint::black_box(e.eval(&env)); }
    });

    // ════════════════════════════════════════════════════════════════
    println!("\n═══ Experiment 2: huge expressions (500x each) ═══\n");
    // ════════════════════════════════════════════════════════════════

    let env_small: Vec<i64> = vec![1; num_vars as usize];
    let depths = [14, 16, 18, 20];
    let huge_iters = 500_u32;

    for &depth in &depths {
        let huge = make_huge_expr(depth, num_vars);
        let nodes = node_count(&huge);
        println!("  depth={depth}, nodes={nodes}");

        let lean_huge = to_lean(&huge);

        // -- simplify
        let lean_simp = lean_huge.simplify();
        bench(&format!("  Lean simplify (depth {depth})"), huge_iters, || {
            std::hint::black_box(lean_huge.simplify());
        });

        let rust_simp = simplify(&huge);
        bench(&format!("  Rust simplify (depth {depth})"), huge_iters, || {
            std::hint::black_box(simplify(&huge));
        });

        // -- eval
        bench(&format!("  Lean eval array (depth {depth})"), huge_iters, || {
            std::hint::black_box(lean_simp.eval(&env_small));
        });

        bench(&format!("  Lean eval_with closure (depth {depth})"), huge_iters, || {
            std::hint::black_box(lean_simp.eval_with(|i| env_small[i]));
        });

        bench(&format!("  Rust eval (depth {depth})"), huge_iters, || {
            std::hint::black_box(rust_simp.eval(&env_small));
        });

        println!();
    }
}
