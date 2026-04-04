import ArithCircuit.ArithCircuit

open Expr

-- Constructors: Rust calls these to build Expr trees
@[export arith_expr_const]
def mkConst (n : Int) : Expr := .const n

@[export arith_expr_var]
def mkVar (i : @& Nat) : Expr := .var i

@[export arith_expr_add]
def mkAdd (a b : Expr) : Expr := .add a b

@[export arith_expr_mul]
def mkMul (a b : Expr) : Expr := .mul a b

-- Core operations
@[export arith_expr_simplify]
def simplifyExpr (e : @& Expr) : Expr := e.simplify

-- Eval with a Lean closure (Nat → Int). Rust builds the closure via C shim.
@[export arith_expr_eval_fn]
def evalExprFn (env : @& (Nat → Int)) (e : @& Expr) : Int :=
  e.eval env

-- Eval with array-based environment (simpler FFI, no closures needed)
@[export arith_expr_eval]
def evalExpr (env : @& Array Int) (e : @& Expr) : Int :=
  e.eval (fun i => env.getD i 0)

-- Debug printing
@[export arith_expr_to_string]
def exprToString (e : @& Expr) : String := e.toString

-- Array builders (so Rust can construct a Lean Array Int incrementally)
@[export arith_array_mk]
def mkIntArray : Array Int := #[]

@[export arith_array_push]
def pushIntArray (a : Array Int) (v : Int) : Array Int := a.push v
