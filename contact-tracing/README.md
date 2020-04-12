# contact-tracing

This crate implements the apple/google proximity contact tracing.

The version of this implementation is the [initial reference
spec](https://covid19-static.cdn-apple.com/applications/covid19/current/static/contact-tracing/pdf/ContactTracing-CryptographySpecification.pdf)
from April 2020.

## Features

* `chrono`: Adds timestamp operations to all structs (on by default)
* `serde`: Adds serde support (implies `base64`)
* `base64`: Adds base64 encoding/decoding through `Display` and `FromStr`

## Broadcast Example

To broadcast one needs a tracing key and the rolling proximity identifier
(RPI) for a given time.  The RPI is normally created from the daily tracing
key but there is a shortcut to derive it automatically:

```rust
use contact_tracing::{TracingKey, DailyTracingKey, Rpi};

let tkey = TracingKey::unique();
let rpi = Rpi::for_now(&tkey);
```

## Infection Checking Example

Infection checking uses the daily tracing keys directly:

```rust
use contact_tracing::{TracingKey, DailyTracingKey, Rpi};

// normally these would come from the internet somewhere
let tkey = TracingKey::unique();
let dtkey = DailyTracingKey::for_today(&tkey);

for (tin, rpi) in dtkey.iter_rpis().enumerate() {
    // check your database of contacts against the TIN and RPIs generated
    // for each daily tracing key downloaded.  The TIN should be within
    // some reasonable window of the timestamp you captured.
}
```

License: Apache-2.0
