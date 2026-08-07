<!-- cargo-sync-rdme title [[ -->
# topological-sort
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge [[ -->
[![Maintenance: passively-maintained](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg?)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/topological-sort.svg?)](#license)
[![crates.io](https://img.shields.io/crates/v/topological-sort.svg?logo=rust)](https://crates.io/crates/topological-sort)
[![docs.rs](https://img.shields.io/docsrs/topological-sort.svg?logo=docs.rs)](https://docs.rs/topological-sort)
[![Rust: ^1.88.0](https://img.shields.io/badge/rust-^1.88.0-93450a.svg?logo=rust)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![GitHub Actions: CI](https://img.shields.io/github/actions/workflow/status/gifnksm/topological-sort-rs/ci.yml.svg?label=CI&logo=github)](https://github.com/gifnksm/topological-sort-rs/actions/workflows/ci.yml)
[![Codecov](https://img.shields.io/codecov/c/github/gifnksm/topological-sort-rs.svg?label=codecov&logo=codecov)](https://codecov.io/gh/gifnksm/topological-sort-rs)
<!-- cargo-sync-rdme ]] -->

<!-- cargo-sync-rdme rustdoc [[ -->
A data structure for topological sorting.

## Examples

### Modeling Makefile-style dependencies

This example reproduces a small `Makefile`. Each call to [`TopologicalSort::pop_batch`](https://docs.rs/topological-sort/0.3.1/topological_sort/struct.TopologicalSort.html#method.pop_batch)
returns the next batch of files that can be built in parallel.

````Makefile
hello_world: hello_world.o libhello.so
        gcc -o hello_world hello_world.o -lhello

hello_world.o: hello_world.c hello.h
        gcc -c -o hello_world.o hello_world.c
````

````rust
use topological_sort::TopologicalSort;

let mut ts = TopologicalSort::<&str>::new();

ts.add_dependency("hello_world.o", "hello_world");
ts.add_dependency("libhello.so", "hello_world");
ts.add_dependency("hello_world.c", "hello_world.o");
ts.add_dependency("hello.h", "hello_world.o");

// Source inputs with no remaining dependencies are ready first.
let mut first_group = ts.pop_batch::<Vec<_>>();
first_group.sort();
assert_eq!(first_group, ["hello.h", "hello_world.c", "libhello.so"]);

// Building those inputs makes the object file ready.
let mut second_group = ts.pop_batch::<Vec<_>>();
second_group.sort();
assert_eq!(second_group, ["hello_world.o"]);

// Finally, the executable itself becomes ready.
let mut third_group = ts.pop_batch::<Vec<_>>();
third_group.sort();
assert_eq!(third_group, ["hello_world"]);

assert!(ts.pop_batch::<Vec<_>>().is_empty());
````

### Detecting circular dependencies

This example consumes a sort by repeatedly popping ready items. If any items remain afterward,
the remaining subgraph contains a cycle.

````rust
use topological_sort::TopologicalSort;

fn has_circular_dependency(mut ts: TopologicalSort<&str>) -> bool {
    // Remove every item that can be processed.
    ts.pop_iter().for_each(drop);
    // Any remaining items must be blocked by a cycle.
    !ts.is_empty()
}

let mut ts1 = TopologicalSort::<&str>::new();
ts1.add_dependency("scissors", "rock");
ts1.add_dependency("paper", "scissors");
ts1.add_dependency("rock", "paper");

let mut ts2 = TopologicalSort::<&str>::new();
ts2.add_dependency("grass", "zebra");
ts2.add_dependency("zebra", "lion");

assert!(has_circular_dependency(ts1));
assert!(!has_circular_dependency(ts2));
````

### Processing items one at a time

This example repeatedly calls [`TopologicalSort::pop`](https://docs.rs/topological-sort/0.3.1/topological_sort/struct.TopologicalSort.html#method.pop) to process items as soon as each next
item becomes ready.

````rust
use topological_sort::TopologicalSort;

let mut ts = TopologicalSort::<&str>::new();
ts.add_dependency("parse", "analyze");
ts.add_dependency("analyze", "compile");

while let Some(item) = ts.pop() {
    process(item);
}

````

### Using `TopologicalSort` in a task scheduler

[`TopologicalSort`](https://docs.rs/topological-sort/0.3.1/topological_sort/struct.TopologicalSort.html) can serve as the dependency tracker inside a task
scheduler. [`TopologicalSort::peek_batch`](https://docs.rs/topological-sort/0.3.1/topological_sort/struct.TopologicalSort.html#method.peek_batch) returns all tasks whose
prerequisites are satisfied, and [`TopologicalSort::remove`](https://docs.rs/topological-sort/0.3.1/topological_sort/struct.TopologicalSort.html#method.remove) marks a
completed task as done, which may make more tasks ready.

Because [`TopologicalSort::peek_batch`](https://docs.rs/topological-sort/0.3.1/topological_sort/struct.TopologicalSort.html#method.peek_batch) does not remove tasks, a scheduler
also needs to track which ready tasks are already running so it does not
start them twice.

````rust
use std::collections::HashSet;

use topological_sort::TopologicalSort;

type Task = String;

fn run_scheduler(tasks: TopologicalSort<Task>) {
    let mut remaining_tasks = tasks;
    let mut running_tasks = HashSet::<Task>::new();

    while !remaining_tasks.is_empty() {
        // `peek_batch()` returns every task whose prerequisites are
        // satisfied, including tasks that are already running.
        let runnable_or_running_tasks = remaining_tasks.peek_batch();

        let runnable_tasks = runnable_or_running_tasks
            .filter(|task| !running_tasks.contains(*task))
            .cloned()
            .collect::<Vec<Task>>();

        if !runnable_tasks.is_empty() {
            start_tasks(&runnable_tasks);
            running_tasks.extend(runnable_tasks);
        }

        // Wait for one running task to finish, then mark it complete.
        let completed_task = wait_for_task_completion(&running_tasks);
        remaining_tasks.remove(&completed_task);
        running_tasks.remove(&completed_task);
    }
}
````
<!-- cargo-sync-rdme ]] -->

[Documentation](https://docs.rs/topological-sort)

## How to use?

Add this to your `Cargo.toml`:

```toml
[dependencies]
topological-sort = "0.3.1"
```

## Minimum supported Rust version (MSRV)

The minimum supported Rust version is **Rust 1.88.0**.

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
