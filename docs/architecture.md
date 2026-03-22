# RuCAS Architecture

## Scope

This crate is being shaped as a focused scalar CAS, not as a full SymPy clone.
The first target surface is:

- symbolic expressions
- differentiation
- simplification
- factorization
- integration

Everything else should stay out of the core until those engine areas have clean
interfaces and stable invariants.

## Design Rules

1. The AST stays small and immutable.
2. Canonicalization happens in constructors, not as an afterthought in every
   algorithm.
3. Algorithms live in separate engines. The expression type should not become a
   dumping ground for symbolic behavior.
4. When an operation cannot finish, it returns an explicit unevaluated node such
   as `Derivative` or `Integral`.
5. No global mutable state, hidden caches, or singleton-heavy design.

## SymPy Lessons

Ideas worth keeping:

- normalized n-ary `Add` and `Mul`
- explicit unevaluated forms
- layered integrators with ordered heuristics
- a rewrite-oriented simplification story

Ideas worth rejecting in Rust:

- a huge dynamic class hierarchy with behavior spread across many subclasses
- "everything is a method on the base expression object"
- implicit global behavior that is hard to reason about in tests
- a monolithic `simplify()` implementation that becomes the home for every rule

## Module Layout

```text
src/
  context.rs        Engine-wide execution knobs
  core/
    expr.rs         Immutable AST handle and node definitions
    number.rs       Numeric atom abstraction
    symbol.rs       Symbols and assumptions
    function.rs     Builtin and user-defined function heads
    canon.rs        Canonical constructors for Add/Mul/Pow
    visit.rs        Shared traversal and folding infrastructure
  rewrite/
    mod.rs          Rule trait and fixpoint rewrite engine
  simplify/
    mod.rs          Simplifier facade over rewrite pipelines
  factor/
    mod.rs          Strategy-based factorization engine
  diff/
    mod.rs          Differentiation engine
  integrate/
    mod.rs          Strategy-based integration engine
```

## Why This Split Works

`core` owns data and invariants.

`rewrite` owns local transformations.

`simplify` owns pass ordering and policy.

`factor` owns factorization strategies and fallback behavior.

`diff` owns derivative semantics.

`integrate` owns heuristic search and step reporting.

That separation matters because differentiation, simplification, factorization,
and integration all want different traversal and fallback behavior, even when
they share the same expression tree.

## Core Invariants

- `Add` and `Mul` are flattened.
- additive and multiplicative identities are removed.
- numeric atoms are collected during construction.
- algorithms are allowed to assume constructors maintain those invariants.

The current implementation keeps canonicalization intentionally modest. Later we
can strengthen it with coefficient collection, power combination, and exact
ordering without changing the public module boundaries.

## Integration Direction

Integration should not be a single giant function. The intended model is an
ordered strategy list:

1. trivial/base cases
2. linearity splitting
3. table rules
4. substitution heuristics
5. integration by parts
6. polynomial and rational methods
7. fallback to unevaluated integral

Each strategy should be independently testable and should be able to attach a
short step trace.

## Differentiation Direction

Differentiation can be more direct than integration, but it should still be
implemented in an engine module rather than as ad hoc behavior on every node.
That keeps rules centralized and allows different policies later:

- fast derivative without simplification
- derivative with local cleanup
- derivative with full simplify pass

## Simplification Direction

Avoid a catch-all `simplify everything somehow` implementation early.
Prefer explicit pipelines of named rewrite passes, for example:

1. normalization
2. constant folding
3. algebraic identities
4. power rules
5. trig/exponential rules
6. fixpoint loop with budget

This makes regressions easier to isolate and performance easier to reason about.

## Factorization Direction

Factorization should not be hidden inside `simplify()`. It should have a small,
explicit engine with ordered strategies, for example:

1. content and common-factor extraction
2. supported polynomial-shape detection
3. special-form recognition such as difference of squares
4. fallback to the original expression when no sound factorization applies

That keeps the supported surface narrow, testable, and easy to extend without
turning simplification into a catch-all algebra module.

## Likely Next Steps

1. Add a parser/printer boundary instead of constructing expressions manually.
2. Implement factorization strategies for common factors and supported
   polynomials.
3. Implement derivative rules for `Add`, `Mul`, `Pow`, and a few builtin
   functions.
4. Add the first simplification rule set around constants and powers.
5. Introduce the first integration strategies for constants, powers, and
   linearity.
