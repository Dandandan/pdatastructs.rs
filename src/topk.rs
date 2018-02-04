use countminsketch::CountMinSketch;
use std::collections::{BTreeSet, HashMap};
use std::collections::hash_map::Entry;
use std::cmp::Ordering;
use std::hash::Hash;
use std::rc::Rc;

struct TreeEntry<T>
where
    T: Eq + Ord,
{
    obj: Rc<T>,
    n: usize,
}

impl<T> Clone for TreeEntry<T>
where
    T: Eq + Ord,
{
    fn clone(&self) -> Self {
        TreeEntry {
            obj: Rc::clone(&self.obj),
            n: self.n,
        }
    }
}

impl<T> PartialEq for TreeEntry<T>
where
    T: Eq + Ord,
{
    fn eq(&self, other: &TreeEntry<T>) -> bool {
        self.obj == other.obj
    }
}

impl<T> Eq for TreeEntry<T>
where
    T: Eq + Ord,
{
}

impl<T> PartialOrd for TreeEntry<T>
where
    T: Eq + Ord,
{
    fn partial_cmp(&self, other: &TreeEntry<T>) -> Option<Ordering> {
        match self.n.cmp(&other.n) {
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Less => Some(Ordering::Less),
            Ordering::Equal => Some(self.obj.cmp(&other.obj)),
        }
    }
}

impl<T> Ord for TreeEntry<T>
where
    T: Eq + Ord,
{
    fn cmp(&self, other: &TreeEntry<T>) -> Ordering {
        match self.n.cmp(&other.n) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.obj.cmp(&other.obj),
        }
    }
}

/// Top-K implementation.
///
/// This data structure keeps the `k` most frequent data points of a stream.
pub struct TopK<T>
where
    T: Clone + Eq + Hash + Ord,
{
    cms: CountMinSketch,
    obj2count: HashMap<Rc<T>, usize>,
    tree: BTreeSet<TreeEntry<T>>,
    k: usize,
}

impl<T> TopK<T>
where
    T: Clone + Eq + Hash + Ord,
{
    /// Create new Top-K data structure.
    ///
    /// - `k`: number of elements to remember
    /// - `cms`: CountMinSketch that is used for guessing the frequency of elements not currently
    ///   hold. Refer to its documentation about parameter selection.
    pub fn new(k: usize, cms: CountMinSketch) -> TopK<T> {
        TopK {
            cms: cms,
            obj2count: HashMap::new(),
            tree: BTreeSet::new(),
            k: k,
        }
    }

    /// Observe a data point.
    pub fn add(&mut self, obj: T) {
        // create Rc so we can use obj in multiple data structs
        let rc = Rc::new(obj);

        // always increase CountMinSketch counts
        self.cms.add(&rc);

        // check if entry is in top K
        let size = self.obj2count.len();
        let mut update_obj2count: Option<(usize, Rc<T>)> = None;
        match self.obj2count.entry(Rc::clone(&rc)) {
            Entry::Occupied(mut o) => {
                // it is => increase exact counter
                let mut n = o.get_mut();
                *n += 1;

                // update tree data
                let mut entry = TreeEntry {
                    obj: Rc::clone(&rc),
                    n: *n - 1,
                };
                self.tree.remove(&entry);
                entry.n += 1;
                self.tree.insert(entry);
            }
            Entry::Vacant(v) => {
                // it's not => check capicity
                if size < self.k {
                    // space left => add to top k
                    v.insert(1);
                    self.tree.insert(TreeEntry {
                        obj: Rc::clone(&rc),
                        n: 1,
                    });
                } else {
                    // not enough space => check if it would be a top k element
                    let count = self.cms.query_point(&rc);
                    let min: TreeEntry<T> = (*self.tree.iter().next().unwrap()).clone();
                    if count > min.n {
                        // => kick out minimal element of top k
                        self.tree.remove(&min);
                        self.tree.insert(TreeEntry {
                            obj: Rc::clone(&rc),
                            n: count,
                        });

                        // delay obj2count changes, because it is currently borrowed
                        update_obj2count = Some((count, min.obj));
                    }
                }
            }
        }

        // trick the borrow checker
        if let Some((count, obj)) = update_obj2count {
            self.obj2count.insert(rc, count);
            self.obj2count.remove(&obj);
        }
    }

    /// Returns top-k values.
    ///
    /// The result may contain less than `k` values if less than `k` unique data points where
    /// observed.
    pub fn values(&self) -> Vec<T> {
        self.tree.iter().map(|x| (*x.obj).clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use countminsketch::CountMinSketch;
    use super::TopK;

    #[test]
    fn add() {
        let cms = CountMinSketch::<usize>::with_params(10, 20);
        let mut tk = TopK::new(2, cms);

        for i in 0..5 {
            tk.add(i);
        }
        for _ in 0..100 {
            tk.add(99);
        }
        for _ in 0..100 {
            tk.add(100);
        }
        for i in 0..5 {
            tk.add(i);
        }
        assert_eq!(tk.values(), vec![99, 100]);
    }
}
