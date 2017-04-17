# Tokio-JSONRPC

[![Travis Build Status](https://api.travis-ci.org/vorner/tokio-serde-cbor.png?branch=master)](https://travis-ci.org/vorner/tokio-serde-cbor)
[![AppVeyor Build status](https://ci.appveyor.com/api/projects/status/omgsa9hhwd5cpmmc/branch/master?svg=true)](https://ci.appveyor.com/project/vorner/tokio-serde-cbor/branch/master)

This rust crate integrates the
[`serde-cbor`](https://crates.io/crates/serde-cbor) into a codec (`Decoder` and
`Encoder`) of [`tokio-io`](https://crates.io/crates/tokio-io). This allows
turning an async read/write into a stream and sink of objects.

The API documentation can be found [here](https://docs.rs/tokio-serde-cbor).

## Status

The API is not formally stabilized and may change. But the crate itself is
small and there's probably not much space for such changes.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms
or conditions.
