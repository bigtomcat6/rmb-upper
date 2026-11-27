# rmb-upper

Convert RMB/CNY amounts into Chinese uppercase financial text.

这个项目现在由两层组成：

1. **Pure Rust library**: 可直接被 Rust backend、CLI、WASM 或其他 binding 复用。
2. **N-API wrapper**: 保留原来的 Node.js 原生扩展能力。

## Rust usage

The recommended API is `format_cents`, because it represents money as an integer number of cents and avoids floating-point precision issues.

```rust
use rmb_upper::format_cents;

fn main() {
  let value = format_cents(12_345).unwrap();
  assert_eq!(value, "壹佰贰拾叁元肆角伍分");
}
```

For decimal strings, use `format_decimal`:

```rust
use rmb_upper::format_decimal;

assert_eq!(format_decimal("123.45").unwrap(), "壹佰贰拾叁元肆角伍分");
assert_eq!(format_decimal("0.01").unwrap(), "零元壹分");
```

For cross-language bindings or platforms where large integers may lose precision, use `format_cents_str`:

```rust
use rmb_upper::format_cents_str;

assert_eq!(format_cents_str("12345").unwrap(), "壹佰贰拾叁元肆角伍分");
```

The `f64` adapter exists for compatibility only:

```rust
use rmb_upper::format_f64;

assert_eq!(format_f64(123.45).unwrap(), "壹佰贰拾叁元肆角伍分");
```

Prefer the cents or string APIs for financial code.

## Supported range

The maximum supported integer part is:

```text
9999兆9999亿9999万9999
```

The maximum supported amount in cents is exposed as `MAX_CENTS`.

## Errors

The Rust APIs return `Result<String, FormatError>`. Possible error variants include:

- `NegativeAmount`
- `NonFiniteAmount`
- `InvalidFormat`
- `TooManyDecimalPlaces`
- `TooLarge`

## Node.js / N-API usage

The original `formatNumber(number)` API is still available for compatibility. It accepts a JavaScript number and rounds to the nearest cent.

```ts
import { formatNumber } from 'rmb-upper'

formatNumber(123.45) // "壹佰贰拾叁元肆角伍分"
```

For deterministic formatting, prefer string-based APIs:

```ts
import { formatCents, formatDecimal } from 'rmb-upper'

formatCents('12345') // "壹佰贰拾叁元肆角伍分"
formatDecimal('123.45') // "壹佰贰拾叁元肆角伍分"
```

## Building

### Rust library

```bash
cargo build
cargo test
```

### N-API wrapper

```bash
pnpm i
pnpm build
```

The N-API build compiles the Rust core in `bindings/node` into a Node.js native addon.

## Testing

```bash
cargo test
```

For JavaScript tests:

```bash
pnpm test:js
```

## Related projects

- [napi-rs](https://github.com/napi-rs/napi-rs)
