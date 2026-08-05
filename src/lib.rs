//! A data structure for topological sorting.
//!
//! # Examples
//!
//! ## Modeling Makefile-style dependencies
//!
//! This example reproduces a small `Makefile`. Each call to [`TopologicalSort::pop_batch`]
//! returns the next batch of files that can be built in parallel.
//!
//! ```Makefile
//! hello_world: hello_world.o libhello.so
//!         gcc -o hello_world hello_world.o -lhello
//!
//! hello_world.o: hello_world.c hello.h
//!         gcc -c -o hello_world.o hello_world.c
//! ```
//!
//! ```rust
//! use topological_sort::TopologicalSort;
//!
//! let mut ts = TopologicalSort::<&str>::new();
//!
//! ts.add_dependency("hello_world.o", "hello_world");
//! ts.add_dependency("libhello.so", "hello_world");
//! ts.add_dependency("hello_world.c", "hello_world.o");
//! ts.add_dependency("hello.h", "hello_world.o");
//!
//! // Source inputs with no remaining dependencies are ready first.
//! let mut first_group = ts.pop_batch::<Vec<_>>();
//! first_group.sort();
//! assert_eq!(first_group, ["hello.h", "hello_world.c", "libhello.so"]);
//!
//! // Building those inputs makes the object file ready.
//! let mut second_group = ts.pop_batch::<Vec<_>>();
//! second_group.sort();
//! assert_eq!(second_group, ["hello_world.o"]);
//!
//! // Finally, the executable itself becomes ready.
//! let mut third_group = ts.pop_batch::<Vec<_>>();
//! third_group.sort();
//! assert_eq!(third_group, ["hello_world"]);
//!
//! assert!(ts.pop_batch::<Vec<_>>().is_empty());
//! ```
//!
//! ## Detecting circular dependencies
//!
//! This example consumes a sort by repeatedly popping ready items. If any items remain afterward,
//! the remaining subgraph contains a cycle.
//!
//! ```rust
//! use topological_sort::TopologicalSort;
//!
//! fn has_circular_dependency(mut ts: TopologicalSort<&str>) -> bool {
//!     // Remove every item that can be processed.
//!     ts.pop_iter().for_each(drop);
//!     // Any remaining items must be blocked by a cycle.
//!     !ts.is_empty()
//! }
//!
//! let mut ts1 = TopologicalSort::<&str>::new();
//! ts1.add_dependency("scissors", "rock");
//! ts1.add_dependency("paper", "scissors");
//! ts1.add_dependency("rock", "paper");
//!
//! let mut ts2 = TopologicalSort::<&str>::new();
//! ts2.add_dependency("grass", "zebra");
//! ts2.add_dependency("zebra", "lion");
//!
//! assert!(has_circular_dependency(ts1));
//! assert!(!has_circular_dependency(ts2));
//! ```
//!
//! ## Processing items one at a time
//!
//! This example repeatedly calls [`TopologicalSort::pop`] to process items as soon as each next
//! item becomes ready.
//!
//! ```rust
//! use topological_sort::TopologicalSort;
//!
//! # fn process(_item: &str) {}
//! let mut ts = TopologicalSort::<&str>::new();
//! ts.add_dependency("parse", "analyze");
//! ts.add_dependency("analyze", "compile");
//!
//! # let mut processed = Vec::new();
//! while let Some(item) = ts.pop() {
//!     process(item);
//! #   processed.push(item);
//! }
//!
//! # assert_eq!(processed, ["parse", "analyze", "compile"]);
//! ```
//!
//! ## Using `TopologicalSort` in a task scheduler
//!
//! [`TopologicalSort`] can serve as the dependency tracker inside a task
//! scheduler. [`TopologicalSort::peek_batch`] returns all tasks whose
//! prerequisites are satisfied, and [`TopologicalSort::remove`] marks a
//! completed task as done, which may make more tasks ready.
//!
//! Because [`TopologicalSort::peek_batch`] does not remove tasks, a scheduler
//! also needs to track which ready tasks are already running so it does not
//! start them twice.
//!
//! ```rust
//! use std::collections::HashSet;
//!
//! use topological_sort::TopologicalSort;
//!
//! type Task = String;
//!
//! # fn start_tasks(_tasks: &[Task]) {}
//! # fn wait_for_task_completion(_running_tasks: &HashSet<Task>) -> Task { todo!() }
//! fn run_scheduler(tasks: TopologicalSort<Task>) {
//!     let mut remaining_tasks = tasks;
//!     let mut running_tasks = HashSet::<Task>::new();
//!
//!     while !remaining_tasks.is_empty() {
//!         // `peek_batch()` returns every task whose prerequisites are
//!         // satisfied, including tasks that are already running.
//!         let runnable_or_running_tasks = remaining_tasks.peek_batch();
//!
//!         let runnable_tasks = runnable_or_running_tasks
//!             .filter(|task| !running_tasks.contains(*task))
//!             .cloned()
//!             .collect::<Vec<Task>>();
//!
//!         if !runnable_tasks.is_empty() {
//!             start_tasks(&runnable_tasks);
//!             running_tasks.extend(runnable_tasks);
//!         }
//!
//!         // Wait for one running task to finish, then mark it complete.
//!         let completed_task = wait_for_task_completion(&running_tasks);
//!         remaining_tasks.remove(&completed_task);
//!         running_tasks.remove(&completed_task);
//!     }
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet, hash_map},
    fmt,
    hash::Hash,
    iter::{FromIterator, FusedIterator},
};

#[derive(Clone, Debug)]
struct Node<T> {
    num_prec: usize,
    succ: HashSet<T>,
}

impl<T> Node<T>
where
    T: Eq + Hash,
{
    fn new() -> Node<T> {
        Node {
            num_prec: 0,
            succ: HashSet::new(),
        }
    }

    fn is_ready(&self) -> bool {
        self.num_prec == 0
    }
}

/// A data structure for topological sorting.
///
/// See the [crate-level documentation](crate) for examples.
#[derive(Clone)]
pub struct TopologicalSort<T> {
    nodes: HashMap<T, Node<T>>,
}

impl<T> Default for TopologicalSort<T> {
    fn default() -> TopologicalSort<T> {
        TopologicalSort {
            nodes: HashMap::new(),
        }
    }
}

impl<T> fmt::Debug for TopologicalSort<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(self.nodes.iter().map(|(k, dep)| (k, &dep.succ)))
            .finish()
    }
}

impl<T> TopologicalSort<T>
where
    T: Clone + Eq + Hash,
{
    /// Creates a new empty `TopologicalSort`.
    ///
    /// See the [crate-level documentation](crate) for examples.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of remaining items in the `TopologicalSort`.
    ///
    /// This counts all remaining items, including those that are not yet ready to pop.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the `TopologicalSort` contains no remaining items.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Registers a dependency from `prec` to `succ`.
    ///
    /// This means that `succ` depends on `prec`, so `prec` must be popped or removed
    /// before `succ` becomes ready.
    ///
    /// Returns `true` if this dependency link was newly added, or `false` if it was
    /// already present.
    ///
    /// ```rust
    /// use topological_sort::TopologicalSort;
    ///
    /// let mut ts = TopologicalSort::new();
    /// assert!(ts.add_dependency("compile", "link"));
    ///
    /// assert_eq!(ts.pop(), Some("compile"));
    /// assert_eq!(ts.pop(), Some("link"));
    /// ```
    pub fn add_dependency<P, S>(&mut self, prec: P, succ: S) -> bool
    where
        P: Into<T>,
        S: Into<T>,
    {
        let prec = prec.into();
        let succ = succ.into();

        let prec_node = self.nodes.entry(prec).or_insert_with(Node::new);
        if !prec_node.succ.insert(succ.clone()) {
            // Already registered
            return false;
        }
        self.nodes.entry(succ).or_insert_with(Node::new).num_prec += 1;
        true
    }

    /// Registers a dependency link.
    ///
    /// This means that `link.succ` depends on `link.prec`, so `link.prec` must be
    /// popped or removed before `link.succ` becomes ready.
    ///
    /// Returns `true` if this dependency link was newly added, or `false` if it was
    /// already present.
    ///
    /// ```rust
    /// use topological_sort::{DependencyLink, TopologicalSort};
    ///
    /// let mut ts = TopologicalSort::new();
    /// assert!(ts.add_link(DependencyLink {
    ///     prec: "compile",
    ///     succ: "link"
    /// }));
    ///
    /// assert_eq!(ts.pop(), Some("compile"));
    /// assert_eq!(ts.pop(), Some("link"));
    /// ```
    pub fn add_link(&mut self, link: DependencyLink<T>) -> bool {
        self.add_dependency(link.prec, link.succ)
    }

    /// Inserts an item, without adding any dependencies from or to it.
    ///
    /// Returns `true` if the item was not already present, or `false` otherwise.
    ///
    /// ```rust
    /// use topological_sort::TopologicalSort;
    ///
    /// let mut ts = TopologicalSort::new();
    /// assert!(ts.insert("standalone"));
    /// assert!(!ts.insert("standalone"));
    ///
    /// assert_eq!(ts.pop(), Some("standalone"));
    /// ```
    pub fn insert<U>(&mut self, item: U) -> bool
    where
        U: Into<T>,
    {
        match self.nodes.entry(item.into()) {
            hash_map::Entry::Vacant(e) => {
                e.insert(Node::new());
                true
            }
            hash_map::Entry::Occupied(_) => false,
        }
    }

    /// Removes one item that does not depend on any other remaining item and returns it, or
    /// `None` if there is no such item.
    ///
    /// If `pop` returns `None` and `len` is not 0, the remaining items contain a cycle.
    ///
    /// ```rust
    /// use topological_sort::TopologicalSort;
    ///
    /// let mut ts = TopologicalSort::new();
    /// ts.add_dependency("a", "b");
    ///
    /// assert_eq!(ts.pop(), Some("a"));
    /// assert_eq!(ts.pop(), Some("b"));
    /// assert_eq!(ts.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        let (item, node) = self.nodes.extract_if(|_, node| node.is_ready()).next()?;
        for succ in node.succ {
            if let Some(succ_node) = self.nodes.get_mut(&succ) {
                succ_node.num_prec -= 1;
            }
        }
        Some(item)
    }

    /// Returns an iterator that repeatedly calls [`pop`](Self::pop).
    ///
    /// Each call to [`Iterator::next`] removes one item from the sort.
    ///
    /// The iterator ends when the sort becomes empty or when no item can be popped because the
    /// remaining items contain a cycle.
    ///
    /// ```rust
    /// use topological_sort::TopologicalSort;
    ///
    /// let mut ts = TopologicalSort::new();
    /// ts.add_dependency(1, 2);
    /// ts.add_dependency(2, 3);
    ///
    /// let mut it = ts.pop_iter();
    /// assert_eq!(Some(1), it.next());
    /// assert_eq!(Some(2), it.next());
    /// drop(it);
    ///
    /// assert_eq!(Some(3), ts.pop());
    /// ```
    pub fn pop_iter(&mut self) -> PopIter<'_, T> {
        PopIter { ts: self }
    }

    /// Removes all items that do not depend on any other remaining item at the time of the call
    /// and returns them, or an empty collection if there are no such items.
    ///
    /// Unlike [`pop_iter`](Self::pop_iter), this removes only the current batch of ready items. If
    /// removing those items makes more items ready, they are returned by the next call to
    /// `pop_batch`, not the current one.
    ///
    /// The returned items are in arbitrary order.
    ///
    /// If `pop_batch` returns an empty collection and `len` is not 0, the remaining items contain
    /// a cycle.
    ///
    /// ```rust
    /// use topological_sort::TopologicalSort;
    ///
    /// let mut ts = TopologicalSort::<i32>::new();
    /// ts.add_dependency(1, 3);
    /// ts.add_dependency(2, 3);
    ///
    /// let mut ready = ts.pop_batch::<Vec<_>>();
    /// ready.sort_unstable();
    /// assert_eq!(ready, [1, 2]);
    ///
    /// assert_eq!(ts.pop_batch::<Vec<_>>(), [3]);
    /// ```
    pub fn pop_batch<R>(&mut self) -> R
    where
        R: Default + Extend<T>,
    {
        let (items, nodes) = self
            .nodes
            .extract_if(|_, node| node.is_ready())
            .collect::<(R, Vec<_>)>();
        for node in nodes {
            for succ in node.succ {
                if let Some(succ_node) = self.nodes.get_mut(&succ) {
                    succ_node.num_prec -= 1;
                }
            }
        }
        items
    }

    /// Returns a reference to one item that does not depend on any other remaining item, or
    /// `None` if there is no such item.
    ///
    /// ```rust
    /// use topological_sort::TopologicalSort;
    ///
    /// let mut ts = TopologicalSort::new();
    /// ts.add_dependency("a", "b");
    ///
    /// assert_eq!(ts.peek(), Some(&"a"));
    /// assert_eq!(ts.len(), 2);
    /// assert_eq!(ts.pop(), Some("a"));
    /// ```
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        let (item, _) = self.nodes.iter().find(|&(_, node)| node.is_ready())?;
        Some(item)
    }

    /// Returns an iterator over references to all items that do not depend on any other remaining
    /// item at the time of the call.
    ///
    /// The iterator yields no items if there are no such items. This inspects only the current
    /// batch of ready items.
    ///
    /// The returned items are in arbitrary order.
    ///
    /// ```rust
    /// use topological_sort::TopologicalSort;
    ///
    /// let mut ts = TopologicalSort::<i32>::new();
    /// ts.add_dependency(1, 3);
    /// ts.add_dependency(2, 3);
    ///
    /// let mut ready = ts.peek_batch().copied().collect::<Vec<_>>();
    /// ready.sort_unstable();
    /// assert_eq!(ready, [1, 2]);
    /// assert_eq!(ts.len(), 3);
    /// ```
    pub fn peek_batch(&self) -> PeekBatch<'_, T> {
        PeekBatch {
            iter: self.nodes.iter(),
        }
    }

    /// Returns an iterator visiting all remaining items in arbitrary order.
    ///
    /// This includes items that are not yet ready because they are blocked by unresolved
    /// dependencies or cycles.
    pub fn items(&self) -> Items<'_, T> {
        Items {
            iter: self.nodes.keys(),
        }
    }

    /// Returns a consuming iterator visiting all remaining items in arbitrary order.
    ///
    /// This includes items that are not yet ready because they are blocked by unresolved
    /// dependencies or cycles.
    pub fn into_items(self) -> IntoItems<T> {
        IntoItems {
            iter: self.nodes.into_keys(),
        }
    }

    /// Removes the specified item if it does not depend on any other remaining item and returns
    /// it.
    ///
    /// Returns `None` if the item is not present or if it still depends on another remaining item.
    ///
    /// Removing the item also removes its outgoing dependency links, which may make some successor
    /// items ready.
    ///
    /// ```rust
    /// use topological_sort::TopologicalSort;
    ///
    /// let mut ts = TopologicalSort::new();
    /// ts.add_dependency("a", "b");
    ///
    /// assert_eq!(ts.remove("b"), None);
    /// assert_eq!(ts.remove("a"), Some("a"));
    /// assert_eq!(ts.remove("b"), Some("b"));
    /// ```
    pub fn remove<Q>(&mut self, item: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let node = self.nodes.get(item)?;
        if !node.is_ready() {
            return None;
        }
        let (item, node) = self.nodes.remove_entry(item)?;
        for succ in node.succ {
            if let Some(succ_node) = self.nodes.get_mut(succ.borrow()) {
                succ_node.num_prec -= 1;
            }
        }
        Some(item)
    }
}

/// A dependency link between two items in a sort.
#[derive(Copy, Clone, Debug)]
pub struct DependencyLink<T> {
    /// The item that `succ` depends on.
    pub prec: T,
    /// The item that depends on `prec`.
    pub succ: T,
}

impl<T> FromIterator<DependencyLink<T>> for TopologicalSort<T>
where
    T: Clone + Eq + Hash,
{
    fn from_iter<I>(iter: I) -> TopologicalSort<T>
    where
        I: IntoIterator<Item = DependencyLink<T>>,
    {
        let mut ts = TopologicalSort::new();
        ts.extend(iter);
        ts
    }
}

impl<T> Extend<DependencyLink<T>> for TopologicalSort<T>
where
    T: Clone + Eq + Hash,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = DependencyLink<T>>,
    {
        for link in iter {
            self.add_link(link);
        }
    }
}

/// An iterator over items popped from a [`TopologicalSort`].
///
/// This struct is created by [`TopologicalSort::pop_iter`].
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct PopIter<'a, T> {
    ts: &'a mut TopologicalSort<T>,
}

impl<T> Iterator for PopIter<'_, T>
where
    T: Clone + Eq + Hash,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.ts.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.ts.len()))
    }
}

impl<T> FusedIterator for PopIter<'_, T> where T: Clone + Eq + Hash {}

/// An iterator over all items in a [`TopologicalSort`] that do not depend on any other remaining
/// item at the time of the call.
///
/// This struct is created by [`TopologicalSort::peek_batch`].
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct PeekBatch<'a, T> {
    iter: hash_map::Iter<'a, T, Node<T>>,
}

impl<'a, T> Iterator for PeekBatch<'a, T>
where
    T: Clone + Eq + Hash,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let (item, _) = self.iter.find(|&(_, node)| node.is_ready())?;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.iter.len()))
    }
}

impl<T> FusedIterator for PeekBatch<'_, T> where T: Clone + Eq + Hash {}

/// An iterator over all remaining items in a [`TopologicalSort`].
///
/// This struct is created by [`TopologicalSort::items`].
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Items<'a, T> {
    iter: hash_map::Keys<'a, T, Node<T>>,
}

impl<'a, T> Iterator for Items<'a, T>
where
    T: Clone + Eq + Hash,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> ExactSizeIterator for Items<'_, T> where T: Clone + Eq + Hash {}

impl<T> FusedIterator for Items<'_, T> where T: Clone + Eq + Hash {}

/// An iterator that consumes a [`TopologicalSort`] and yields all remaining items.
///
/// This struct is created by [`TopologicalSort::into_items`].
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntoItems<T> {
    iter: hash_map::IntoKeys<T, Node<T>>,
}

impl<T> Iterator for IntoItems<T>
where
    T: Clone + Eq + Hash,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> ExactSizeIterator for IntoItems<T> where T: Clone + Eq + Hash {}
impl<T> FusedIterator for IntoItems<T> where T: Clone + Eq + Hash {}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn add_dependency_returns_true_if_new_dependency_link_created() {
        let mut ts = TopologicalSort::<&str>::new();
        assert!(ts.add_dependency("stone", "sharp"));
        assert_eq!(ts.len(), 2);
        assert!(!ts.add_dependency("stone", "sharp"));
        assert_eq!(ts.len(), 2);
        assert!(ts.add_dependency("sharp", "paper"));
        assert_eq!(ts.len(), 3);
        assert!(!ts.add_dependency("sharp", "paper"));
        assert_eq!(ts.len(), 3);
        assert!(ts.add_dependency("paper", "stone"));
        assert_eq!(ts.len(), 3);
        assert!(!ts.add_dependency("paper", "stone"));
        assert_eq!(ts.len(), 3);
    }

    #[test]
    fn add_link_returns_true_if_new_dependency_link_created() {
        let mut ts = TopologicalSort::<&str>::new();
        assert!(ts.add_link(DependencyLink {
            prec: "stone",
            succ: "sharp",
        }));
        assert!(!ts.add_link(DependencyLink {
            prec: "stone",
            succ: "sharp",
        }));
        assert_eq!(ts.len(), 2);
    }

    #[test]
    fn pop_iter_iterates_all_items_in_topological_order() {
        let mut ts = TopologicalSort::<i32>::new();
        ts.add_dependency(1, 2);
        ts.add_dependency(2, 3);
        ts.add_dependency(3, 4);
        ts.add_dependency(4, 5);
        ts.add_dependency(5, 6);
        let mut it = ts.pop_iter();
        assert_eq!(Some(1), it.next());
        assert_eq!(Some(2), it.next());
        assert_eq!(Some(3), it.next());
        assert_eq!(Some(4), it.next());
        assert_eq!(Some(5), it.next());
        assert_eq!(Some(6), it.next());
        assert_eq!(None, it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn pop_iter_stops_on_a_cycle() {
        let mut ts = TopologicalSort::<i32>::new();
        ts.add_dependency(1, 2);
        ts.add_dependency(2, 3);
        ts.add_dependency(3, 4);
        ts.add_dependency(4, 5);
        ts.add_dependency(5, 5);
        ts.add_dependency(5, 6);
        let mut it = ts.pop_iter();
        assert_eq!(Some(1), it.next());
        assert_eq!(Some(2), it.next());
        assert_eq!(Some(3), it.next());
        assert_eq!(Some(4), it.next());
        assert_eq!(None, it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn pop_batch_returns_all_currently_ready_items() {
        fn check(result: &[i32], ts: &mut TopologicalSort<i32>) {
            let l = ts.len();
            let mut v = ts.pop_batch::<Vec<_>>();
            v.sort_unstable();
            assert_eq!(result, &v[..]);
            assert_eq!(l - result.len(), ts.len());
        }

        let mut ts = TopologicalSort::new();
        ts.add_dependency(7, 11);
        assert_eq!(2, ts.len());
        ts.add_dependency(7, 8);
        assert_eq!(3, ts.len());
        ts.add_dependency(5, 11);
        assert_eq!(4, ts.len());
        ts.add_dependency(3, 8);
        assert_eq!(5, ts.len());
        ts.add_dependency(3, 10);
        assert_eq!(6, ts.len());
        ts.add_dependency(11, 2);
        assert_eq!(7, ts.len());
        ts.add_dependency(11, 9);
        assert_eq!(8, ts.len());
        ts.add_dependency(11, 10);
        assert_eq!(8, ts.len());
        ts.add_dependency(8, 9);
        assert_eq!(8, ts.len());

        check(&[3, 5, 7], &mut ts);
        check(&[8, 11], &mut ts);
        check(&[2, 9, 10], &mut ts);
        check(&[], &mut ts);
    }

    #[test]
    fn self_dependency_blocks_the_remaining_element() {
        let mut ts = TopologicalSort::<&str>::new();
        ts.add_dependency("stone", "sharp");
        ts.add_dependency("sharp", "sharp");
        ts.add_dependency("sharp", "water");
        assert_eq!(ts.len(), 3);
        assert_eq!(ts.pop(), Some("stone"));
        assert_eq!(ts.len(), 2);
        assert_eq!(ts.pop(), None);
    }

    #[test]
    fn pop_returns_none_when_remaining_elements_are_cyclic() {
        let mut ts = TopologicalSort::new();
        ts.add_dependency("stone", "sharp");

        ts.add_dependency("bucket", "hole");
        ts.add_dependency("hole", "straw");
        ts.add_dependency("straw", "axe");
        ts.add_dependency("axe", "sharp");
        ts.add_dependency("sharp", "water");
        ts.add_dependency("water", "bucket");
        assert_eq!(ts.pop(), Some("stone"));
        assert!(ts.pop().is_none());
    }

    #[test]
    fn add_link_can_create_a_cycle_that_blocks_remaining_elements() {
        let mut ts = TopologicalSort::<&str>::new();

        ts.add_link(DependencyLink {
            prec: "omelet",
            succ: "egg",
        });
        ts.add_link(DependencyLink {
            prec: "egg",
            succ: "chicken",
        });
        ts.add_link(DependencyLink {
            prec: "chicken",
            succ: "egg",
        });
        assert_eq!(ts.len(), 3);
        assert_eq!(ts.pop(), Some("omelet"));
        assert_eq!(ts.pop(), None);
    }

    #[test]
    fn remove_removes_item_only_if_exists_and_ready() {
        let mut ts = TopologicalSort::<&str>::new();
        ts.add_dependency("a", "b");
        ts.add_dependency("b", "c");
        ts.add_dependency("c", "d");

        assert!(ts.remove("x").is_none());
        assert!(ts.remove("c").is_none());
        assert_eq!(ts.remove("a").unwrap(), "a");
        assert!(ts.remove("c").is_none());
        assert_eq!(ts.remove("b").unwrap(), "b");
        assert_eq!(ts.remove("c").unwrap(), "c");
    }

    #[test]
    fn items_and_into_items_iterate_all_remaining_items() {
        let mut ts = TopologicalSort::<&str>::new();
        ts.add_dependency("a", "b");
        ts.add_dependency("b", "c");
        ts.add_dependency("c", "d");

        let mut items = ts.items().copied().collect::<Vec<_>>();
        items.sort_unstable();
        assert_eq!(items, ["a", "b", "c", "d"]);

        let mut into_items = ts.into_items().collect::<Vec<_>>();
        into_items.sort_unstable();
        assert_eq!(into_items, ["a", "b", "c", "d"]);
    }

    #[quickcheck]
    fn quickcheck_topological_sort_invariants(n: usize, edges: Vec<(usize, usize)>) {
        use std::collections::{HashMap, HashSet};

        let n = n.clamp(1, 1000);
        let mut marked = vec![false; n];
        let edges = edges
            .into_iter()
            .map(|(x, y)| (x % n, y % n))
            .collect::<Vec<_>>();
        let mut deps = HashMap::new();
        let mut toposort = TopologicalSort::<usize>::new();

        for i in 0..n {
            deps.insert(i, HashSet::new());
            assert!(toposort.insert(i));
        }

        for (op, inp) in edges.iter().map(|(x, y)| (y, x)) {
            let inps = deps.get_mut(op).unwrap();
            inps.insert(*inp);
        }

        let deps = deps;
        for (inp, op) in edges {
            toposort.add_dependency(inp, op);
        }
        while let Some(x) = toposort.pop() {
            for dep in &deps[&x] {
                assert!(marked[*dep]);
            }
            marked[x] = true;
        }

        if toposort.is_empty() {
            assert!(marked.into_iter().all(|x| x));
        } else {
            let dep_fixed = {
                let mut ret = (0..n)
                    .map(|i| (i, HashSet::new()))
                    .collect::<HashMap<_, _>>();
                let mut new_to_add = deps;

                while !new_to_add.is_empty() {
                    for (k, v) in new_to_add.drain() {
                        let inps = ret.get_mut(&k).unwrap();
                        inps.extend(v.into_iter());
                    }
                    for (k, vs) in &ret {
                        for k2 in vs {
                            for v2 in &ret[k2] {
                                if !vs.contains(v2) {
                                    new_to_add
                                        .entry(*k)
                                        .or_insert_with(HashSet::new)
                                        .insert(*v2);
                                }
                            }
                        }
                    }
                }

                ret
            };

            assert!(dep_fixed.into_iter().any(|(op, deps)| deps.contains(&op)));
        }
    }
}
