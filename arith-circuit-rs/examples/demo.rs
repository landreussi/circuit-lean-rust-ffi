use arith_circuit::{Expr, init};

fn main() {
    init();

    // Build: (x0 + 0) * (1 * x1)
    let expr = Expr::mul(
        &Expr::add(&Expr::var(0), &Expr::constant(0)),
        &Expr::mul(&Expr::constant(1), &Expr::var(1)),
    );
    println!("Original:   {expr}");

    let simplified = expr.simplify();
    println!("Simplified: {simplified}");

    // Eval with x0=3, x1=5
    let result = expr.eval(&[3, 5]);
    let result_s = simplified.eval(&[3, 5]);
    println!("eval original:   {result}");
    println!("eval simplified: {result_s}");
    assert_eq!(result, result_s);

    // Constant folding: (2 + 3) * x0
    let expr2 = Expr::mul(
        &Expr::add(&Expr::constant(2), &Expr::constant(3)),
        &Expr::var(0),
    );
    let simplified2 = expr2.simplify();
    println!("\nOriginal:   {expr2}");
    println!("Simplified: {simplified2}");
    println!("eval(x0=7): {}", simplified2.eval(&[7]));

    // eval_with: pass a Rust closure directly to Lean
    let result = simplified.eval_with(|i| match i {
        0 => 10,
        1 => 20,
        _ => 0,
    });
    println!("\neval_with(x0=10, x1=20): {result}");
    assert_eq!(result, 200);

    println!("\nAll assertions passed!");
}
