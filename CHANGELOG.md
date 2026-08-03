# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

* Added `CHANGELOG.md`.

### Changed

* **(Breaking Change)** `TopologicalSort::add_dependency()` and `TopologicalSort::add_link()` now return `true` when they add a new edge and `false` when the edge already existed.
* **(Breaking Change)** Removed `impl From<(T, T)> for DependencyLink<T>`.
  The tuple order was the inverse of `TopologicalSort::add_dependency(prec,
  succ)` and `DependencyLink { prec, succ }`, which made it easy to invert an
  edge by mistake.

  Migrate tuple-based construction to explicit field names:
  * Replace `ts.add_link((succ, prec).into())` with
    `ts.add_link(DependencyLink { prec, succ })`.
  * Replace `DependencyLink::from((succ, prec))` with
    `DependencyLink { prec, succ }`.
  * Replace `.map(DependencyLink::from)` on `(succ, prec)` tuples with
    `.map(|(succ, prec)| DependencyLink { prec, succ })`.
* Raised the minimum supported Rust version to Rust 1.85.0.
* Updated project maintenance and tooling.
  * Added regression tests covering self-dependencies and cycles created with `DependencyLink`.
  * Added repository-wide configuration in `.editorconfig`, `.gitattributes`, `.markdownlintignore`, `codecov.yml`, `deny.toml`, and `justfile`, expanded Cargo metadata in `Cargo.toml` for linting and README synchronization, and moved release automation into `release.toml`.
  * Split GitHub Actions automation into dedicated workflows in `.github/workflows/ci.yml`, `.github/workflows/cd.yml`, `.github/workflows/audit.yml`, and `.github/workflows/update-deps.yml`, updated `.github/dependabot.yml`, and expanded checks across Linux, macOS, and Windows.
  * Started tracking `Cargo.lock`.
  * Changed the repository's default branch from `master` to `main` and updated related automation and README badges.

### Fixed

* Updated copyright and license statements.

## [0.2.2] - 2022-07-18

### Changed

* Marked the crate as passively maintained.
* Updated release automation.
  * Moved release-time configuration into Cargo metadata.
  * Removed the tag-triggered GitHub Actions publish workflow.

## [0.2.1] - 2022-07-17

### Fixed

* Fixed the manifest to use the supported `rust-version` key for declaring the MSRV.

## [0.2.0] - 2022-07-17

### Added

* Implemented `Default` for `TopologicalSort<T>`, allowing construction with `TopologicalSort::default()`.

### Changed

* Migrated the crate to Rust 2018 and declared Rust 1.43.1 as the minimum supported Rust version.
* Expanded project automation and test coverage.
  * Added property-based tests covering ordering and cycle detection behavior.
  * Added GitHub Actions CI, release automation, and Dependabot configuration.
  * Removed the Travis CI configuration.

## [0.1.0] - 2018-01-03

### Changed

* Updated CI configuration and made ignored return values explicit to satisfy newer Rust warnings.

## [0.0.10] - 2017-09-10

### Changed

* Changed `TopologicalSort::insert()` and `TopologicalSort::add_dependency()` to accept arguments implementing `Into<T>`.

## [0.0.9] - 2017-05-30

### Added

* Added the `DependencyLink<T>` type and `TopologicalSort::add_link()` for registering dependency edges as values.
* Implemented `From<(T, T)>` for `DependencyLink<T>`, allowing dependency links to be created from tuples.
* Implemented `FromIterator<DependencyLink<T>>` for `TopologicalSort<T>`, allowing a sorter to be built from an iterator of dependency links.
* Implemented `Clone` for `TopologicalSort<T>` and `Copy`, `Clone`, and `Debug` for `DependencyLink<T>`.

### Changed

* Documentation moved to docs.rs.
* Updated project tooling.
  * Switched CI from `travis-cargo` helpers to direct Cargo commands.

### Removed

* Removed Travis-based GitHub Pages documentation publishing.

## [0.0.8] - 2017-05-01

### Added

* Implemented `fmt::Debug` for `TopologicalSort<T>`.
* Added `TopologicalSort::peek()` and `TopologicalSort::peek_all()` to inspect items that are ready to pop without removing them.

## [0.0.7] - 2016-08-08

### Added

* Added `TopologicalSort::insert()` to register elements that have no dependencies.
* Implemented `FromIterator<T>` for partially ordered element types, deriving dependency edges from `partial_cmp()`.

## [0.0.6] - 2016-01-11

### Added

* Added Apache-2.0 as an alternative license alongside MIT.

### Changed

* Renamed the crate for use as `topological_sort` in code and documentation.
* Updated project tooling for newer Rust versions.
  * Migrated CI to `travis-cargo`, expanded testing to nightly, beta, and stable Rust, and enabled documentation and coverage reporting.

### Fixed

* Updated documentation examples to use the correct `topological_sort` crate name.

## [0.0.5] - 2015-03-29

### Changed

* Updated the crate for `rustc 1.0.0-nightly (199bdcfef 2015-03-26)`.
  * Removed the now-unneeded `std_misc` feature gate.
* This removed the last nightly-only feature gate, so this release became buildable on stable Rust once Rust 1.0 shipped on 2015-05-15.

## [0.0.4] - 2015-02-21

### Changed

* Updated the crate for `rustc 1.0.0-nightly (522d09dfe 2015-02-19)`.
  * Switched generic bounds back to plain `Hash`.
  * Added the `std_misc` feature gate.

## [0.0.3] - 2015-01-09

### Changed

* Updated the crate for `rustc 1.0.0-dev (20bce4481 2015-01-09 04:14:53 +0000)`.
  * **(Breaking Change)** Changed `TopologicalSort::len()` to return `usize` instead of `uint`.
  * **(Breaking Change)** Changed the public trait bounds on `TopologicalSort<T>` and its `Iterator` implementation from `Hash` to `Hash<Hasher>`.

## [0.0.2] - 2015-01-07

### Changed

* Updated the crate for `rustc 1.0.0-dev (9e4e524e0 2015-01-07 05:31:23 +0000)`.
  * Removed the crate-level `associated_types` feature gate after associated types no longer required opting in.

## [0.0.1] - 2015-01-06

* First release

<!-- next-url -->
[Unreleased]: https://github.com/gifnksm/topological-sort-rs/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/gifnksm/topological-sort-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/gifnksm/topological-sort-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/gifnksm/topological-sort-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.10...v0.1.0
[0.0.10]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.9...v0.0.10
[0.0.9]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/gifnksm/topological-sort-rs/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/gifnksm/topological-sort-rs/releases/tag/v0.0.1
