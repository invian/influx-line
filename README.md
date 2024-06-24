# Influx Line Protocol

This crate implements
[InfluxDB Line Protocol v2](https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol).

## Rationale

The main focus is convenience and safety, and only then performance.
Unfortunately, other libraries did not fulfill our needs immediately.

This is how we addressed those concerns:

- Safety: Each type with its own restrictions.
- Convenience: built-in traits `FromStr` and `Display` for those types.
- Performance: reducing extra copying and allocations when possible.

We simply like Line Protocol, but we do not need batch parsing all the time.
The expected use case for this crate is parsing a single line at once.
It may not be optimal, and may not fit everyone.

### Comparison to other crates

This package:

- Intended for parsing lines one-by-one.
  - Imperative style leaning towards "OOP".
- API and usage as per our company development policies.

[`influxdb_line_protocol`](https://docs.rs/influxdb-line-protocol/latest/influxdb_line_protocol/)

- Seemingly intended for batch parsing.
  - Uses [`nom`](https://crates.io/crates/nom).
- Not the most convenient API for us.

[`rinfluxdb`](https://docs.rs/rinfluxdb/latest/rinfluxdb/index.html)

- Nice builder-style API for instantiating lines.
- Not intended for raw string parsing since it's a query client.
