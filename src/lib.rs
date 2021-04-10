use std::cmp;

use std::collections::HashMap;
use std::collections::HashSet;

use std::cmp::Eq;
use std::hash::Hash;

use unicode_normalization::UnicodeNormalization;

#[cfg(test)]
mod tests {
    #[test]
    fn integer_keys_invert() {
        let mut baar = crate::InvertIndex::<u32>::default();
        baar.insert(69, "boof");
        baar.insert(420, "foob");

        let mut fooo = crate::InvertIndex::<u32>::default();
        fooo.insert(420, "foob");
        fooo.insert(69, "boof");

        assert_eq!(baar.index, fooo.index);
        assert_eq!(baar.query("oof"), fooo.query("oof"));
    }

    #[test]
    fn str_keys_invert() {
        let mut quux = crate::InvertIndex::<&str>::default();
        quux.insert("pleased", "boof");
        quux.insert("blazed", "foob");

        let mut baaz = crate::InvertIndex::<&str>::default();
        baaz.insert("blazed", "foob");
        baaz.insert("pleased", "boof");

        assert_eq!(quux.index, baaz.index);
        assert_eq!(quux.query("oof"), baaz.query("oof"));
    }

    #[test]
    fn integer_keys_merge() {
        let mut baar = crate::MergeIndex::<u32>::default();
        baar.insert(69, "boof");
        baar.insert(420, "foob");

        let mut fooo = crate::MergeIndex::<u32>::default();
        fooo.insert(420, "foob");
        fooo.insert(69, "boof");

        assert_eq!(baar.index, fooo.index);
        assert_eq!(baar.query("oof"), fooo.query("oof"));
    }

    #[test]
    fn str_keys_merge() {
        let mut quux = crate::MergeIndex::<&str>::default();
        quux.insert("pleased", "boof");
        quux.insert("blazed", "foob");

        let mut baaz = crate::MergeIndex::<&str>::default();
        baaz.insert("blazed", "foob");
        baaz.insert("pleased", "boof");

        assert_eq!(quux.index, baaz.index);
        assert_eq!(quux.query("oof"), baaz.query("oof"));
    }

    #[test]
    fn invert_vs_merge() {
        let mut baar = crate::InvertIndex::<u32>::default();
        baar.insert(69, "boof");
        baar.insert(420, "foob");

        let mut fooo = crate::MergeIndex::<u32>::default();
        fooo.insert(420, "foob");
        fooo.insert(69, "boof");

        assert_eq!(baar.query("oof"), fooo.query("oof"));
    }

    #[test]
    fn invert_vs_merge_from() {
        let mut baar = crate::InvertIndex::<u32>::default();
        baar.insert(69, "boof");
        baar.insert(420, "foob");

        let fooo = crate::MergeIndex::from(baar.clone());

        assert_eq!(baar.query("oof"), fooo.query("oof"));
    }

    #[test]
    fn merge_vs_invert_from() {
        let mut baar = crate::MergeIndex::<u32>::default();
        baar.insert(69, "boof");
        baar.insert(420, "foob");

        let fooo = crate::InvertIndex::from(baar.clone());

        assert_eq!(baar.query("oof"), fooo.query("oof"));
    }
}

#[derive(Clone)]
pub struct MergeIndex<T> {
    depth: usize,
    index: HashMap<T, HashSet<String>>,
}

#[derive(Clone)]
pub struct InvertIndex<T> {
    depth: usize,
    index: HashMap<String, HashSet<T>>,
}

impl<T> InvertIndex<T>
where
    T: Hash,
    T: Eq,
    T: Copy,
{
    pub fn default() -> Self {
        InvertIndex::new(6)
    }

    pub fn new(depth: usize) -> Self {
        InvertIndex {
            depth: depth,
            index: HashMap::<String, HashSet<T>>::new(),
        }
    }

    pub fn insert(&mut self, key: T, index_string: &str) {
        let gs = mkgrams(index_string, self.depth);
        for gram in gs.iter() {
            let entry = self
                .index
                .entry(gram.to_owned())
                .or_insert_with(HashSet::<T>::new);
            entry.insert(key);
        }
    }

    pub fn query(&self, query_string: &str) -> Vec<T> {
        let gs = mkgrams(query_string, self.depth);
        let mut accum_map = HashMap::<T, u32>::new();
        for gram in gs.iter() {
            match self.index.get(gram) {
                Some(ks) => {
                    for k in ks.iter() {
                        *accum_map.entry(*k).or_insert(0) += 1;
                    }
                }
                None => {}
            };
        }
        let mut inter = accum_map.iter().collect::<Vec<(&T, &u32)>>();
        inter.sort_by(|a, b| a.1.cmp(&b.1).reverse());
        inter.iter().map(|x| *x.0).collect::<Vec<T>>()
    }
}

impl<T> MergeIndex<T>
where
    T: Hash,
    T: Eq,
    T: Copy,
{
    pub fn default() -> Self {
        MergeIndex::new(6)
    }

    pub fn new(depth: usize) -> Self {
        MergeIndex {
            depth: depth,
            index: HashMap::<T, HashSet<String>>::new(),
        }
    }

    pub fn insert(&mut self, key: T, index_string: &str) {
        let gs = mkgrams(index_string, self.depth);
        self.index.insert(
            key,
            match self.index.get(&key) {
                Some(kg) => gs.union(&kg).map(|x| String::from(x)).collect(),
                None => gs,
            },
        );
    }

    pub fn query(&self, query_string: &str) -> Vec<T> {
        let gs = mkgrams(query_string, self.depth);
        let mut inter = self
            .index
            .iter()
            .map(|x| (*x.0, x.1.intersection(&gs).count()))
            .filter(|x| x.1 != 0)
            .collect::<Vec<(T, usize)>>();
        inter.sort_by(|a, b| a.1.cmp(&b.1).reverse());
        inter.iter().map(|x| x.0).collect::<Vec<T>>()
    }
}

impl<T> From<MergeIndex<T>> for InvertIndex<T>
where
    T: Hash,
    T: Eq,
    T: Copy,
{
    fn from(merge_index: MergeIndex<T>) -> Self {
        let mut index = HashMap::<String, HashSet<T>>::new();
        for (key, gs) in merge_index.index {
            for gram in gs.iter() {
                let entry = index
                    .entry(gram.to_owned())
                    .or_insert_with(HashSet::<T>::new);
                entry.insert(key);
            }
        }
        InvertIndex {
            depth: merge_index.depth,
            index: index,
        }
    }
}

impl<T> From<InvertIndex<T>> for MergeIndex<T>
where
    T: Hash,
    T: Eq,
    T: Copy,
{
    fn from(invert_index: InvertIndex<T>) -> Self {
        let mut index = HashMap::<T, HashSet<String>>::new();
        for (gram, ks) in invert_index.index {
            for key in ks.iter() {
                let entry = index.entry(*key).or_insert_with(HashSet::<String>::new);
                entry.insert(gram.clone());
            }
        }
        MergeIndex {
            depth: invert_index.depth,
            index: index,
        }
    }
}

/// Make a set of char n-grams from a string, to a given depth.
///
/// # Example
/// ```rust
/// # use indexrs::mkgrams;
/// let foo = mkgrams("foobarbazfoobar", 6);
/// let bar = mkgrams("barbazfoobarbaz", 6);
/// assert_eq!(foo, bar);
/// ```
pub fn mkgrams(s: &str, depth: usize) -> HashSet<String> {
    let mut gs = HashSet::<String>::new();
    let norm: String = s.nfd().collect::<String>().to_lowercase();
    let len: usize = norm.len();
    let rdepth: usize = cmp::min(depth, len);
    for i in 0..len {
        for j in 0..rdepth {
            if let Some(slicette) = norm.get(i..=(i + j + 1)) {
                gs.insert(String::from(slicette));
            }
        }
    }
    gs
}
