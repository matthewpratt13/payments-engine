# Payments Engine

A streaming payments engine that reads transactions from a CSV file, applies them
in chronological order, and writes client account balances to stdout.

```bash
cargo run -- transactions.csv > accounts.csv
```

The input path is the only CLI argument. Account output is written to stdout.
Errors are written to stderr.

## Input

CSV columns: `type`, `client`, `tx`, `amount`.

| `type` | `amount` | Effect |
| --- | --- | --- |
| `deposit` | required | Credit available funds |
| `withdrawal` | required | Debit available funds when the balance is sufficient |
| `dispute` | ignored | Hold a previously settled deposit |
| `resolve` | ignored | Release a held deposit back to available |
| `chargeback` | ignored | Reverse a held deposit and freeze the account |

`client` is a `u16`. `tx` is a globally unique `u32`. Amounts use up to four
decimal places. Header and field whitespace is ignored. Transactions are applied
in file order.

## Output

CSV columns: `client`, `available`, `held`, `total`, `locked`.

`total` is always `available + held`. Rows are sorted by client ID. Amounts are
written with four decimal places.

## Assumptions

These behaviours are not fully specified by the challenge brief:

- Only deposits can be disputed. Withdrawals are not retained, so a dispute,
  resolve, or chargeback that refers to a withdrawal is ignored.
- A dispute holds the original deposit amount even if those funds were later
  withdrawn. `available` may therefore become negative; `total` is still
  `available + held`.
- After a chargeback the account stays locked and every later transaction for
  that client is ignored.
- Duplicate deposit IDs are ignored. The first deposit with a given `tx` wins.
- Zero or negative amounts are ignored.
- A missing amount on a deposit or withdrawal is a fatal input error. An amount
  present on dispute, resolve, or chargeback is ignored.
- An unknown `type` is a fatal CSV parse error.
- Invalid references, client mismatches, and illegal state transitions are
  ignored and do not abort processing.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
```
