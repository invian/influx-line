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

We simply like Line Protocol, so we made our own solution that fits our use cases.
It may not be optimal, and may not fit everyone.
