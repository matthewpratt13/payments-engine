# AGENTS.md

## Project

This repository contains a small Rust payment-processing engine.

The application reads transactions from a CSV file, processes them chronologically, and writes final client account balances as CSV to stdout.

Run with:

```bash
cargo run -- transactions.csv > accounts.csv
```

## Goals

Prioritize, in order:

1. Correctness
2. Maintainability and clarity
3. Robust error handling
4. Efficient memory usage
5. Minimal dependencies

Keep the implementation simple and idiomatic Rust. Avoid unnecessary abstractions, traits, async code, concurrency, or framework dependencies.

## Architecture

Keep the domain logic independent of CSV/file I/O:

```text
CSV input → parser → payment engine → account state → CSV output
```

The engine should process transactions one at a time rather than loading the entire input file into memory.

Recommended domain concepts:

* `ClientId = u16`
* `TransactionId = u32`
* Exact decimal/fixed-point money representation; do not use `f32`/`f64`
* `Account` containing `available`, `held`, and `locked`
* `total` should be derived as `available + held`
* Successful deposits must retain enough transaction information to support later disputes, resolves, and chargebacks
* Represent transaction lifecycle using an enum/state machine rather than multiple boolean flags

Use `HashMap` for client and transaction lookup unless there is a concrete reason to choose another structure.

## Transaction rules

Implement the transaction semantics defined by the challenge:

* Deposits increase available funds.
* Withdrawals decrease available funds only when sufficient available funds exist.
* Failed withdrawals must not modify the account.
* Disputes move the referenced transaction's amount from available to held.
* Resolves move disputed funds from held back to available.
* Chargebacks remove disputed funds from held and lock the client account.
* Invalid dispute/resolve/chargeback references should be ignored.
* Validate that referenced transactions belong to the client performing the operation.
* Invalid transaction state transitions should not modify account state.
* A locked account must remain locked.

Document any interpretation of ambiguous requirements in `README.md`.

## Input/output

Use `serde` and `csv` for CSV serialization/deserialization.

Input must tolerate whitespace and decimal values with up to four decimal places.

Do not write logging or diagnostics to stdout because stdout is the CSV output. Use stderr if diagnostics are genuinely necessary.

Output columns:

```text
client,available,held,total,locked
```

Prefer deterministic client ordering in output even though row ordering is not semantically significant.

## Errors

Distinguish between:

* malformed/unreadable input, which should return an application error; and
* valid transactions that cannot be applied because of business rules, which should generally be ignored.

Do not use `unwrap()`/`expect()` on external input or other recoverable runtime conditions.

## Testing

Add unit tests for account operations and transaction state transitions.

Include integration tests covering the CLI and CSV input/output.

At minimum test:

* deposits
* successful and failed withdrawals
* disputes
* resolves
* chargebacks
* invalid references
* invalid state transitions
* multiple clients
* transaction ownership validation
* locked accounts
* decimal precision
* whitespace handling
* disputes after funds have already been withdrawn

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
```

Keep tests focused on observable behaviour and accounting invariants.

## Code style

Prefer small functions with explicit responsibilities.

Prefer domain types and enums over strings and loosely related flags.

Avoid premature optimization, but preserve streaming input and O(1)-expected-time transaction/client lookup.

Do not add dependencies unless they materially simplify the implementation or improve correctness.

Keep the repository free of challenge-specific confidential/proprietary material.
