# RuCAS Scope

## Purpose

RuCAS is a Rust-first symbolic computer algebra system focused on scalar
symbolic mathematics. The goal is not to clone SymPy wholesale. The goal is to
build a maintainable core that can represent symbolic expressions and support
symbolic differentiation, simplification, factorization, and integration with
explicit, testable algorithms.

## Product Boundary

The planned first release boundary is a scalar CAS with:

- immutable symbolic expression trees
- exact integer and rational arithmetic
- canonical algebraic construction for core operators
- symbolic differentiation
- symbolic simplification through explicit rewrite passes
- symbolic factorization through explicit, bounded algorithms
- symbolic integration through ordered strategies
- parsing and printing for the supported expression language
- tests, examples, and basic step reporting for nontrivial transformations

## In Scope

### 1. Core Expression System

The core expression model should cover:

- numbers
- symbols
- builtin function calls
- n-ary addition
- n-ary multiplication
- exponentiation
- explicit unevaluated derivative nodes
- explicit unevaluated integral nodes

The tree must support arbitrarily nested expressions such as:

- powers within powers
- fractions represented via negative powers or normalized multiplicative form
- nested function calls
- products of sums
- sums of rational expressions

### 2. Exact Scalar Arithmetic

The numeric core should support:

- arbitrary-precision integers
- arbitrary-precision rationals
- normalized sign handling
- deterministic equality and hashing
- exact arithmetic for constructor-time normalization and symbolic algorithms

Floating-point numerics are not a priority for the first milestone.

### 3. Canonicalization

Constructor-time normalization should handle:

- flattening nested `Add` and `Mul`
- removing additive and multiplicative identities
- collecting numeric terms and factors
- stable operand ordering
- normalizing trivial powers such as `x^1` and `x^0`

Later passes may add stronger canonical forms such as coefficient collection and
power combination, but the constructor invariants must stay small and reliable.

### 4. Simplification

Simplification is in scope as a pass pipeline, not as a single opaque routine.
The first meaningful simplification surface should include:

- constant folding
- algebraic identity elimination
- fraction and power cleanup
- limited trig, exponential, and logarithmic rewrite rules
- fixpoint rewriting with a configurable budget
- explicit step traces where useful

### 5. Factorization

Factorization is in scope as an explicit algebraic engine, not as a hidden side
effect of simplification. The first meaningful factorization surface should
include:

- exact extraction of numeric content
- common-factor extraction from sums
- factoring supported univariate polynomials over integers and rationals
- recognition of simple special forms such as difference of squares
- explicit unchanged fallback when a case is unsupported
- direct tests for each supported factor family

General multivariate factorization and algebraic-extension factorization are
outside the first milestone.

### 6. Differentiation

Differentiation should support:

- constants and symbols
- sums
- products
- powers
- chain rule on builtin functions
- explicit unevaluated results when a case is unsupported
- optional post-derivative simplification

The initial builtin function set should be small and deliberate, for example:

- `sin`
- `cos`
- `exp`
- `log`

### 7. Integration

Integration should be strategy-driven and ordered. The first supported surface
should include:

- constants
- powers
- linearity over sums
- constant-factor extraction
- small table-driven builtin function cases
- limited substitution heuristics
- limited integration by parts
- explicit unevaluated fallback when no strategy succeeds

Risch-complete integration is outside the first milestone.

### 8. Parsing and Printing

The project should support a narrow but useful text interface for:

- parsing supported expressions
- pretty-printing expressions back to text
- stable debug-oriented formatting for tests
- round-trip coverage for the supported subset

### 9. Testing and Documentation

The first milestone should include:

- unit tests for each algebraic rule group
- golden tests for parser, printer, and canonicalization
- cross-module regression tests
- architecture and scope documents
- examples demonstrating supported differentiation, simplification,
  factorization, and integration

## Stretch Scope

These items are acceptable if the core milestone is already stable:

- richer symbol assumptions
- more builtin functions
- more factorization families
- simplification step traces exposed in the public API
- a small REPL or command runner
- lightweight performance benchmarks

## Out of Scope

The following are explicitly out of scope for the initial milestone:

- matrices and tensors
- sets, logic, geometry, combinatorics, or number theory modules
- differential equation solvers
- general equation solving
- code generation
- general multivariate polynomial factorization
- factorization over algebraic number fields
- Groebner-basis-style polynomial machinery
- symbolic series expansions
- piecewise-heavy calculus machinery
- complete assumption propagation
- full special-function coverage
- Risch-complete or Meijer-G style integration systems
- symbolic domains beyond scalar expressions

## Quality Bar

The scope is only considered complete when:

- the public expression model is stable enough to add algorithms without
  reshaping the AST
- each supported rule family has direct tests
- unsupported cases degrade to explicit unevaluated expressions rather than
  panicking or producing invalid algebra
- normalization is deterministic
- the parser, printer, simplifier, factorizer, differentiator, and integrator
  agree on the same core invariants

## Delivery Phases

### Phase 1: Core and Representation

- exact numbers
- symbols and functions
- canonical AST construction
- traversal utilities

### Phase 2: Front Door

- parser
- printers
- examples and CLI surface

### Phase 3: Algebraic Engine

- rewrite engine
- simplification passes
- factorization engine and passes

### Phase 4: Calculus Engine

- differentiation
- integration

### Phase 5: Hardening

- regression tests
- documentation
- benchmarks
