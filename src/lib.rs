//! Performs topological sorting.

#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    fmt,
    hash::Hash,
    iter::FromIterator,
};

#[derive(Clone, Debug)]
struct Dependency<T> {
    num_prec: usize,
    succ: HashSet<T>,
}

impl<T: Hash + Eq> Dependency<T> {
    fn new() -> Dependency<T> {
        Dependency {
            num_prec: 0,
            succ: HashSet::new(),
        }
    }
}

/// Performs topological sorting.
#[derive(Clone)]
pub struct TopologicalSort<T> {
    top: HashMap<T, Dependency<T>>,
}

impl<T> Default for TopologicalSort<T> {
    fn default() -> TopologicalSort<T> {
        TopologicalSort {
            top: HashMap::new(),
        }
    }
}

impl<T: Hash + Eq + Clone> TopologicalSort<T> {
    /// Creates new empty `TopologicalSort`.
    ///
    /// ```rust
    /// use topological_sort::TopologicalSort;
    ///
    /// let mut ts = TopologicalSort::<&str>::new();
    /// ts.add_dependency("hello_world.o", "hello_world");
    /// ts.add_dependency("hello_world.c", "hello_world");
    /// ts.add_dependency("stdio.h", "hello_world.o");
    /// ts.add_dependency("glibc.so", "hello_world");
    /// assert_eq!(vec!["glibc.so", "hello_world.c", "stdio.h"], {
    ///     let mut v = ts.pop_all();
    ///     v.sort();
    ///     v
    /// });
    /// assert_eq!(vec!["hello_world.o"], {
    ///     let mut v = ts.pop_all();
    ///     v.sort();
    ///     v
    /// });
    /// assert_eq!(vec!["hello_world"], {
    ///     let mut v = ts.pop_all();
    ///     v.sort();
    ///     v
    /// });
    /// assert!(ts.pop_all().is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> TopologicalSort<T> {
        Self::default()
    }

    /// Returns the number of elements in the `TopologicalSort`.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.top.len()
    }

    /// Returns true if the `TopologicalSort` contains no elements.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.top.is_empty()
    }

    /// Registers the two elements' dependency.
    ///
    /// # Arguments
    ///
    /// * `prec` - The element appears before `succ`. `prec` is depended on by `succ`.
    /// * `succ` - The element appears after `prec`. `succ` depends on `prec`.
    pub fn add_dependency<P, S>(&mut self, prec: P, succ: S) -> bool
    where
        P: Into<T>,
        S: Into<T>,
    {
        let prec = prec.into();
        let succ = succ.into();
        match self.top.entry(prec) {
            Entry::Vacant(e) => {
                let mut dep = Dependency::new();
                dep.succ.insert(succ.clone());
                e.insert(dep);
            }
            Entry::Occupied(e) => {
                if !e.into_mut().succ.insert(succ.clone()) {
                    // Already registered
                    return false;
                }
            }
        }

        match self.top.entry(succ) {
            Entry::Vacant(e) => {
                let mut dep = Dependency::new();
                dep.num_prec += 1;
                e.insert(dep);
            }
            Entry::Occupied(e) => {
                e.into_mut().num_prec += 1;
            }
        }
        true
    }

    /// Registers a dependency link.
    pub fn add_link(&mut self, link: DependencyLink<T>) -> bool {
        self.add_dependency(link.prec, link.succ)
    }

    /// Inserts an element, without adding any dependencies from or to it.
    ///
    /// If the `TopologicalSort` did not have this element present, `true` is returned.
    ///
    /// If the `TopologicalSort` already had this element present, `false` is returned.
    pub fn insert<U>(&mut self, elt: U) -> bool
    where
        U: Into<T>,
    {
        match self.top.entry(elt.into()) {
            Entry::Vacant(e) => {
                let dep = Dependency::new();
                e.insert(dep);
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    /// Removes the item that is not depended on by any other items and returns it, or `None` if
    /// there is no such item.
    ///
    /// If `pop` returns `None` and `len` is not 0, there is cyclic dependencies.
    pub fn pop(&mut self) -> Option<T> {
        self.peek().cloned().inspect(|key| {
            self.remove(key);
        })
    }

    /// Removes all items that are not depended on by any other items and returns it, or empty
    /// vector if there are no such items.
    ///
    /// If `pop_all` returns an empty vector and `len` is not 0, there is cyclic dependencies.
    pub fn pop_all(&mut self) -> Vec<T> {
        let keys = self
            .top
            .iter()
            .filter(|&(_, v)| v.num_prec == 0)
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();
        for k in &keys {
            self.remove(k);
        }
        keys
    }

    /// Return a reference to the first item that does not depend on any other items, or `None` if
    /// there is no such item.
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.top
            .iter()
            .filter(|&(_, v)| v.num_prec == 0)
            .map(|(k, _)| k)
            .next()
    }

    /// Return a vector of references to all items that do not depend on any other items, or an
    /// empty vector if there are no such items.
    #[must_use]
    pub fn peek_all(&self) -> Vec<&T> {
        self.top
            .iter()
            .filter(|&(_, v)| v.num_prec == 0)
            .map(|(k, _)| k)
            .collect::<Vec<_>>()
    }

    fn remove(&mut self, prec: &T) -> Option<Dependency<T>> {
        let result = self.top.remove(prec);
        if let Some(ref p) = result {
            for s in &p.succ {
                if let Some(y) = self.top.get_mut(s) {
                    y.num_prec -= 1;
                }
            }
        }
        result
    }
}

/// A link between two items in a sort.
#[derive(Copy, Clone, Debug)]
pub struct DependencyLink<T> {
    /// The element which is depended upon by `succ`.
    pub prec: T,
    /// The element which depends on `prec`.
    pub succ: T,
}

impl<T: Eq + Hash + Clone> FromIterator<DependencyLink<T>> for TopologicalSort<T> {
    fn from_iter<I>(iter: I) -> TopologicalSort<T>
    where
        I: IntoIterator<Item = DependencyLink<T>>,
    {
        let mut top = TopologicalSort::new();
        for link in iter {
            top.add_link(link);
        }
        top
    }
}

impl<T: Hash + Eq + Clone> Iterator for TopologicalSort<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.pop()
    }
}

impl<T: fmt::Debug> fmt::Debug for TopologicalSort<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(self.top.iter().map(|(k, dep)| (k, &dep.succ)))
            .finish()
    }
}

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
    fn next_returns_elements_in_topological_order() {
        let mut ts = TopologicalSort::<i32>::new();
        ts.add_dependency(1, 2);
        ts.add_dependency(2, 3);
        ts.add_dependency(3, 4);
        assert_eq!(Some(1), ts.next());
        assert_eq!(Some(2), ts.next());
        assert_eq!(Some(3), ts.next());
        assert_eq!(Some(4), ts.next());
        assert_eq!(None, ts.next());
    }

    #[test]
    fn pop_all_returns_all_currently_available_elements() {
        fn check(result: &[i32], ts: &mut TopologicalSort<i32>) {
            let l = ts.len();
            let mut v = ts.pop_all();
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
