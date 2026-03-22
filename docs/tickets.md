# RuCAS Tickets

This backlog decomposes the current scope into explicit work items. Each ticket
should result in a reviewable change with direct tests.

## Ticket Format

- `ID`: stable work item identifier
- `Scope`: which part of `scope.md` it implements
- `Depends on`: prerequisite tickets
- `Deliverable`: concrete implementation target
- `Acceptance`: conditions required to close the ticket

## Backlog

### RUCAS-001: Exact Number Backend

- `Scope`: Exact Scalar Arithmetic
- `Depends on`: none
- `Deliverable`: replace the temporary `i64`-based numeric layer with
  arbitrary-precision integer and rational support.
- `Acceptance`:
  - integers and rationals are normalized deterministically
  - sign normalization is consistent
  - arithmetic used by constructors is exact
  - unit tests cover addition, multiplication, reduction, and equality

### RUCAS-002: Symbol and Assumption Surface

- `Scope`: Core Expression System
- `Depends on`: none
- `Deliverable`: stabilize `Symbol` and `Assumptions` so the public API can
  carry at least basic scalar metadata without affecting core correctness.
- `Acceptance`:
  - symbols have deterministic identity, equality, and display behavior
  - assumptions are serializable as plain data
  - tests cover symbol equality, ordering, and basic assumption storage

### RUCAS-003: Function Head Model

- `Scope`: Core Expression System
- `Depends on`: none
- `Deliverable`: define the builtin and user-defined function-head model used
  by parsing, printing, differentiation, and integration.
- `Acceptance`:
  - builtin functions are represented without stringly-typed special cases
  - unknown named functions remain representable
  - display and equality are stable
  - tests cover builtin and named function heads

### RUCAS-004: Canonical Add and Mul Construction

- `Scope`: Canonicalization
- `Depends on`: RUCAS-001, RUCAS-002
- `Deliverable`: strengthen constructor-time normalization for `Add` and `Mul`.
- `Acceptance`:
  - nested `Add` and `Mul` are flattened
  - zero and one identities are removed correctly
  - numeric terms and factors are collected exactly
  - operand ordering is stable
  - tests cover nested sums, nested products, and mixed numeric-symbolic input

### RUCAS-005: Power and Fraction Normalization

- `Scope`: Canonicalization
- `Depends on`: RUCAS-001, RUCAS-004
- `Deliverable`: normalize powers and establish the project convention for
  fractions as multiplicative inverse forms.
- `Acceptance`:
  - `x^0` and `x^1` normalize correctly
  - negative powers are represented consistently
  - fraction-like expressions print and compare deterministically
  - tests cover nested powers and fractions within fractions

### RUCAS-006: Traversal and Structural Rewrite Utilities

- `Scope`: Core Expression System
- `Depends on`: RUCAS-004, RUCAS-005
- `Deliverable`: stabilize traversal, rebuild, and structural replacement
  helpers used by all algorithms.
- `Acceptance`:
  - recursive folds can rebuild arbitrarily nested expressions
  - visitors traverse all supported node kinds
  - replacement helpers preserve canonical invariants
  - tests cover deep nesting and no-op rewrites

### RUCAS-007: Expression Parser

- `Scope`: Parsing and Printing
- `Depends on`: RUCAS-003, RUCAS-004, RUCAS-005
- `Deliverable`: implement a parser for the supported scalar expression subset.
- `Acceptance`:
  - parser supports numbers, symbols, calls, sums, products, powers, and
    parentheses
  - precedence and associativity are correct
  - invalid input returns structured errors
  - tests cover nested powers, nested fractions, and malformed expressions

### RUCAS-008: Printer Suite

- `Scope`: Parsing and Printing
- `Depends on`: RUCAS-003, RUCAS-004, RUCAS-005
- `Deliverable`: provide stable human-readable and debug-oriented printers.
- `Acceptance`:
  - output is deterministic
  - precedence-aware pretty-printing is correct
  - debug output is suitable for golden tests
  - parser-printer round-trips pass for the supported subset

### RUCAS-009: Rewrite Engine Hardening

- `Scope`: Simplification
- `Depends on`: RUCAS-006
- `Deliverable`: complete the rewrite engine so simplification passes can be
  composed predictably.
- `Acceptance`:
  - ordered rule execution is deterministic
  - fixpoint iteration respects a configurable budget
  - rewrite passes can expose step names
  - tests cover termination and stable results under repeated execution

### RUCAS-010: Simplification Pass 1, Constants and Identities

- `Scope`: Simplification
- `Depends on`: RUCAS-001, RUCAS-004, RUCAS-009
- `Deliverable`: implement the first concrete simplify pass for constant
  folding and algebraic identities.
- `Acceptance`:
  - constant-only subexpressions fold exactly
  - basic identities simplify correctly
  - tests cover additive, multiplicative, and simple power identities

### RUCAS-011: Simplification Pass 2, Powers and Rational Forms

- `Scope`: Simplification
- `Depends on`: RUCAS-005, RUCAS-009, RUCAS-010
- `Deliverable`: implement power cleanup and rational-expression normalization.
- `Acceptance`:
  - nested powers are simplified according to supported rules
  - multiplicative inverse forms are simplified safely
  - fractions within fractions normalize into stable forms where valid
  - tests cover safe and intentionally unsupported cases

### RUCAS-012: Simplification Pass 3, Trig, Exp, and Log Basics

- `Scope`: Simplification
- `Depends on`: RUCAS-003, RUCAS-009, RUCAS-010
- `Deliverable`: add a small, explicit library of builtin-function rewrite
  rules.
- `Acceptance`:
  - the supported rule set is documented
  - rules do not silently apply invalid branch-sensitive rewrites
  - tests cover representative `sin`, `cos`, `exp`, and `log` cases

### RUCAS-023: Factorization Engine and Common-Factor Extraction

- `Scope`: Factorization
- `Depends on`: RUCAS-004, RUCAS-005, RUCAS-006
- `Deliverable`: introduce a dedicated factorization engine with deterministic
  strategy ordering and support exact content extraction plus common-factor
  extraction from sums.
- `Acceptance`:
  - factorization is not implemented as an ad hoc branch inside `simplify`
  - numeric content is extracted exactly
  - common multiplicative factors can be pulled from supported sums
  - unsupported cases return the original expression unchanged
  - tests cover positive, neutral, and unsupported cases

### RUCAS-024: Supported Univariate Polynomial Factorization

- `Scope`: Factorization
- `Depends on`: RUCAS-001, RUCAS-005, RUCAS-023
- `Deliverable`: factor supported univariate polynomials over integers and
  rationals using explicit, documented algorithms.
- `Acceptance`:
  - the supported polynomial subset is documented
  - special forms such as difference of squares are recognized where valid
  - supported low-degree polynomial cases factor deterministically
  - unsupported or ambiguous cases remain unchanged
  - tests cover both successful and declined factorizations

### RUCAS-013: Differentiation Core Rules

- `Scope`: Differentiation
- `Depends on`: RUCAS-004, RUCAS-005, RUCAS-006
- `Deliverable`: implement derivative rules for constants, symbols, sums,
  products, and powers.
- `Acceptance`:
  - product and power rules are correct for the supported subset
  - unsupported cases fall back to explicit unevaluated derivatives
  - tests cover nested algebraic derivatives

### RUCAS-014: Builtin Function Differentiation and Chain Rule

- `Scope`: Differentiation
- `Depends on`: RUCAS-003, RUCAS-013
- `Deliverable`: implement differentiation for the initial builtin function set
  and apply the chain rule.
- `Acceptance`:
  - `sin`, `cos`, `exp`, and `log` derivatives are correct
  - chain rule works on nested function calls
  - tests cover function nesting and mixed algebraic-functional expressions

### RUCAS-015: Post-Derivative Simplification Pipeline

- `Scope`: Differentiation and Simplification
- `Depends on`: RUCAS-010, RUCAS-011, RUCAS-013, RUCAS-014
- `Deliverable`: wire differentiation into the simplify pipeline in a controlled
  way.
- `Acceptance`:
  - derivative output can be requested raw or simplified
  - simplification does not erase unsupported derivative structure
  - tests cover both modes

### RUCAS-016: Integration Strategy 1, Constants, Powers, and Linearity

- `Scope`: Integration
- `Depends on`: RUCAS-004, RUCAS-005, RUCAS-006
- `Deliverable`: implement the first integration strategies for trivial cases.
- `Acceptance`:
  - constants integrate to `c*x`
  - simple powers integrate correctly for the supported subset
  - sums split by linearity
  - constant factors can be extracted
  - unsupported cases return explicit unevaluated integrals

### RUCAS-017: Integration Strategy 2, Builtin Table Rules

- `Scope`: Integration
- `Depends on`: RUCAS-003, RUCAS-016
- `Deliverable`: add a small table of builtin-function antiderivatives.
- `Acceptance`:
  - the supported table is documented
  - builtin cases such as `sin`, `cos`, and `exp` integrate correctly
  - tests cover positive cases and fallback cases

### RUCAS-018: Integration Strategy 3, Substitution Heuristics

- `Scope`: Integration
- `Depends on`: RUCAS-006, RUCAS-017
- `Deliverable`: implement a narrow substitution heuristic layer.
- `Acceptance`:
  - the heuristic only applies when the substitution is structurally justified
  - successful substitutions emit a step trace
  - tests cover nested substitution-style expressions

### RUCAS-019: Integration Strategy 4, Integration by Parts

- `Scope`: Integration
- `Depends on`: RUCAS-017
- `Deliverable`: implement a limited integration-by-parts strategy.
- `Acceptance`:
  - supported cases are explicit and documented
  - the strategy declines unsupported cases cleanly
  - tests cover at least one cyclic or multi-step example

### RUCAS-020: Integration Fallback and Trace Reporting

- `Scope`: Integration
- `Depends on`: RUCAS-016, RUCAS-017, RUCAS-018, RUCAS-019
- `Deliverable`: finalize the integrator fallback contract and step-reporting
  behavior.
- `Acceptance`:
  - failed integrations preserve the original structure in an unevaluated node
  - step traces are stable enough for assertions
  - tests cover solved and deferred integrations

### RUCAS-021: CLI and Examples

- `Scope`: Parsing and Printing, Testing and Documentation
- `Depends on`: RUCAS-007, RUCAS-008, RUCAS-015, RUCAS-020, RUCAS-023,
  RUCAS-024
- `Deliverable`: provide a minimal command-line or demo entrypoint that can
  parse, print, differentiate, simplify, factor, and integrate supported
  expressions.
- `Acceptance`:
  - examples in docs are executable or testable
  - the CLI surface is intentionally narrow
  - tests cover a few end-to-end flows including factorization

### RUCAS-022: Regression Suite and Invariant Tests

- `Scope`: Testing and Documentation
- `Depends on`: RUCAS-007 through RUCAS-024
- `Deliverable`: add a dedicated regression layer for parser, canonicalization,
  simplification, factorization, differentiation, and integration.
- `Acceptance`:
  - regressions can be added as fixed test cases without new harness work
  - invariants such as deterministic printing and idempotent normalization are
    tested
  - deep nested expressions have coverage

### RUCAS-023: Contributor Docs

- `Scope`: Testing and Documentation
- `Depends on`: RUCAS-021, RUCAS-022
- `Deliverable`: document how to add a rewrite rule, derivative rule, or
  integration strategy without violating project invariants.
- `Acceptance`:
  - docs explain module boundaries and invariants clearly
  - docs explain how unsupported cases should fall back
  - docs reference `architecture.md`, `scope.md`, and `tickets.md`

### RUCAS-024: Benchmark and Performance Baseline

- `Scope`: Stretch Scope
- `Depends on`: RUCAS-022
- `Deliverable`: add a small benchmark or profiling baseline for common nested
  expression workloads.
- `Acceptance`:
  - at least a few representative workloads are measured
  - results are reproducible locally
  - no benchmark requires changing the public algebraic model
