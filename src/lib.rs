use std::cmp;

use std::collections::HashMap;
use std::collections::HashSet;

use std::cmp::Eq;
use std::hash::Hash;

use unicode_normalization::UnicodeNormalization;

#[cfg(test)]
mod tests {
    #[test]
    fn integer_keys() {
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
    fn string_keys() {
        let mut quux = crate::InvertIndex::<&str>::default();
        quux.insert("pleased", "boof");
        quux.insert("blazed", "foob");

        let mut baaz = crate::InvertIndex::<&str>::default();
        baaz.insert("blazed", "foob");
        baaz.insert("pleased", "boof");

        assert_eq!(quux.index, baaz.index);
        assert_eq!(quux.query("oof"), baaz.query("oof"));
    }
}

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
