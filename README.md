<!-- cargo-sync-rdme title [[ -->
# topological-sort
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge [[ -->
[![Maintenance: passively-maintained](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg?)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/topological-sort.svg?)](#license)
[![crates.io](https://img.shields.io/crates/v/topological-sort.svg?logo=rust)](https://crates.io/crates/topological-sort)
[![docs.rs](https://img.shields.io/docsrs/topological-sort.svg?logo=docs.rs)](https://docs.rs/topological-sort)
[![Rust: ^1.85.0](https://img.shields.io/badge/rust-^1.85.0-93450a.svg?logo=rust)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![GitHub Actions: CI](https://img.shields.io/github/actions/workflow/status/gifnksm/topological-sort-rs/ci.yml.svg?label=CI&logo=github)](https://github.com/gifnksm/topological-sort-rs/actions/workflows/ci.yml)
[![Codecov](https://img.shields.io/codecov/c/github/gifnksm/topological-sort-rs.svg?label=codecov&logo=codecov)](https://codecov.io/gh/gifnksm/topological-sort-rs)
<!-- cargo-sync-rdme ]] -->

<!-- cargo-sync-rdme rustdoc [[ -->
Performs topological sorting.

````rust
use topological_sort::TopologicalSort;

let mut ts = TopologicalSort::<&str>::new();

ts.add_dependency("hello_world.o", "hello_world");
ts.add_dependency("hello_world.c", "hello_world.o");
ts.add_dependency("stdio.h", "hello_world.o");
ts.add_dependency("glibc.so", "hello_world");

let mut first_group = ts.pop_batch();
first_group.sort();
assert_eq!(first_group, ["glibc.so", "hello_world.c", "stdio.h"]);

let mut second_group = ts.pop_batch();
second_group.sort();
assert_eq!(second_group, ["hello_world.o"]);

let mut third_group = ts.pop_batch();
third_group.sort();
assert_eq!(third_group, ["hello_world"]);

assert!(ts.pop_batch().is_empty());
````
<!-- cargo-sync-rdme ]] -->

[Documentation](https://docs.rs/topological-sort)

## How to use?

Add this to your `Cargo.toml`:

```toml
[dependencies]
topological-sort = "0.2.2"
```

## Minimum supported Rust version (MSRV)

The minimum supported Rust version is **Rust 1.85.0**.

While a crate is pre-release status (0.x.x) it may have its MSRV bumped in a patch release.
Once a crate has reached 1.x, any MSRV bump will be accompanied with a new minor version.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
