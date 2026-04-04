-- ============================================
-- Arithmetic Circuit: AST, Simplification, and Correctness Proof
-- ============================================

/-- An arithmetic expression over integer constants and indexed variables. -/
inductive Expr where
  | const : Int → Expr
  | var   : Nat → Expr
  | add   : Expr → Expr → Expr
  | mul   : Expr → Expr → Expr
  deriving Repr

namespace Expr

/-- Pretty-print an expression. -/
def toString : Expr → String
  | .const n => if n < 0 then s!"({n})" else s!"{n}"
  | .var i   => s!"x{i}"
  | .add a b => s!"({a.toString} + {b.toString})"
  | .mul a b => s!"({a.toString} * {b.toString})"

instance : ToString Expr := ⟨Expr.toString⟩

-- ============================================
-- Evaluation
-- ============================================

/-- Evaluate an expression given a variable assignment. -/
def eval (env : Nat → Int) : Expr → Int
  | .const n => n
  | .var i   => env i
  | .add a b => a.eval env + b.eval env
  | .mul a b => a.eval env * b.eval env

-- ============================================
-- Simplification
-- ============================================

/-- Smart constructor: simplify an addition node.
    Rules: e + 0 = e, 0 + e = e, const + const = const -/
def simplifyAdd (a b : Expr) : Expr :=
  match a, b with
  | .const 0, e        => e
  | e,        .const 0 => e
  | .const m, .const n => .const (m + n)
  | a,        b        => .add a b

/-- Smart constructor: simplify a multiplication node.
    Rules: e * 1 = e, 1 * e = e, const * const = const -/
def simplifyMul (a b : Expr) : Expr :=
  match a, b with
  | .const 1, e        => e
  | e,        .const 1 => e
  | .const m, .const n => .const (m * n)
  | a,        b        => .mul a b

/-- Simplify an expression bottom-up. -/
def simplify : Expr → Expr
  | .const n => .const n
  | .var i   => .var i
  | .add a b => simplifyAdd a.simplify b.simplify
  | .mul a b => simplifyMul a.simplify b.simplify

-- ============================================
-- Correctness: simplification preserves semantics
-- ============================================

theorem simplifyAdd_correct (env : Nat → Int) (a b : Expr) :
    (simplifyAdd a b).eval env = a.eval env + b.eval env := by
  unfold simplifyAdd
  split <;> simp_all [eval]

theorem simplifyMul_correct (env : Nat → Int) (a b : Expr) :
    (simplifyMul a b).eval env = a.eval env * b.eval env := by
  unfold simplifyMul
  split <;> simp_all [eval]

/-- Evaluating a simplified expression gives the same result as the original. -/
theorem simplify_correct (env : Nat → Int) (e : Expr) :
    e.simplify.eval env = e.eval env := by
  induction e with
  | const _ => rfl
  | var _   => rfl
  | add a b iha ihb =>
    unfold simplify
    simp [simplifyAdd_correct, eval, iha, ihb]
  | mul a b iha ihb =>
    unfold simplify
    simp [simplifyMul_correct, eval, iha, ihb]

end Expr

-- ============================================
-- Demo
-- ============================================

open Expr in
/-- (x₀ + 0) * (1 * x₁) — should simplify to x₀ * x₁ -/
def demo : Expr :=
  .mul (.add (.var 0) (.const 0)) (.mul (.const 1) (.var 1))

#eval s!"Before: {demo}"
#eval s!"After:  {demo.simplify}"

/-- x₀ = 3, x₁ = 5 -/
def demoEnv : Nat → Int
  | 0 => 3
  | 1 => 5
  | _ => 0

#eval s!"eval original:   {demo.eval demoEnv}"
#eval s!"eval simplified: {demo.simplify.eval demoEnv}"

-- Step through this in the Infoview to see the proof state at each tactic:
#check @Expr.simplify_correct
