use alloc::vec::Vec;
use compact_str::CompactString;
use core::{borrow::Borrow, hash::Hash};
use fxhash::hash as fxhash;

#[cfg(test)]
use unix_print::unix_println as println;

/// A generic Hash table which keeps insertion history
pub trait HashTable<K: Hash + Eq, V> {
    /// Initializes an empty hash map with a given (fixed) capacity
    fn new(min_capacity: usize) -> Self;

    /// Inserts or replaces an item in the map. Will raise an error if the map is full
    fn insert(&mut self, key: K, value: V) -> Result<(), &'static str>;

    /// Lookup item in the map
    fn get(&self, key: impl Borrow<str>) -> Option<&V>;

    /// Remove an entry from the map
    fn remove(&mut self, key: impl Borrow<str>) -> Option<V>;

    /// Returns the most recent key-value pair that was either inserted or updated and is still present
    fn get_last(&self) -> Option<(&K, &V)>;

    /// Returns the least recent key-value pair that was either inserted or updated and is still present
    fn get_first(&self) -> Option<(&K, &V)>;

    /// Returns the number of map entries
    fn len(&self) -> usize;

    /// Returns the number of map entries
    fn is_empty(&self) -> bool;
}

#[derive(Debug)]
#[repr(u8)]
enum Entry<K, V> {
    Empty,
    /// Occupied contains Key, Value and the index to the ordered vector
    Occupied(K, V, usize),
    Deleted,
}

type SKeyType = CompactString;
type SValueType = u32;

#[derive(Debug)]
pub struct StrHashTable {
    // The map entries
    // @note: Because we need to track latest and first the buckets also hold
    // the index of the element as per insertion order (next field)
    // Keeping an index is a simple way in rust to keep a reference without using cells and
    // other Rust structures which would incur a runtime penalty.
    buckets: Vec<Entry<SKeyType, SValueType>>,
    // The indices per insertion order. Deleted items are set to None, so indices are
    // kept valid and we don't incur the traditional runtime penalty of really removing the items
    by_insertion: Vec<Option<usize>>,
    // the map capacity (cache)
    capacity: usize,
    // Number of map entries
    size: usize,
    // Index of the first inserted still valid. Keep both indices (insertion and hash bucket)
    first: Option<(usize, usize)>,
    // Index of the last inserted still valid
    last: Option<(usize, usize)>,
}

impl HashTable<SKeyType, SValueType> for StrHashTable {
    fn new(min_capacity: usize) -> Self {
        let capacity = min_capacity.next_power_of_two();
        let mut buckets = Vec::with_capacity(capacity);
        buckets.resize_with(capacity, || Entry::Empty);
        Self {
            buckets,
            capacity,
            size: 0,
            by_insertion: Vec::new(),
            first: None,
            last: None,
        }
    }

    fn insert(&mut self, key: SKeyType, value: SValueType) -> Result<(), &'static str> {
        let h = fxhash(&key);
        let max_attempts = (0.75 * (self.capacity as f64)) as usize;
        for i in 0..max_attempts {
            let bucket_i = (h + i) & (self.capacity - 1);
            if let Entry::Occupied(k, v, _) = &mut self.buckets[bucket_i]
                && key == k
            {
                *v = value;
            } else if let Entry::Empty | Entry::Deleted = self.buckets[bucket_i] {
                // Cross reference structures. Bucket contains K,V and insertion index. Insertion tracks bucket index
                #[cfg(test)]
                println!("Adding to bucket {}", bucket_i);
                self.by_insertion.push(Some(bucket_i));
                let insertion_i = self.by_insertion.len() - 1;
                self.buckets[bucket_i] = Entry::Occupied(key, value, insertion_i);
                if self.first.is_none() {
                    self.first = Some((insertion_i, bucket_i));
                }
                self.last = Some((insertion_i, bucket_i));
                self.size += 1;
                return Ok(());
            }
        }
        Err("Could not insert. No sufficient slots")
    }

    // Lookup using linear probing
    fn get(&self, key: impl Borrow<str>) -> Option<&SValueType> {
        let key = key.borrow();
        let h = fxhash(key);
        let max_attempts = (0.75 * (self.capacity as f64)) as usize;
        for i in 0..max_attempts {
            let bucket_i = (h + i) & (self.capacity - 1);
            match &self.buckets[bucket_i] {
                Entry::Occupied(k, value, _insertion_i) if k == key => {
                    #[cfg(test)]
                    println!("Found! Slot: {} order: {}", bucket_i, _insertion_i);
                    return Some(value);
                }
                Entry::Empty => return None,
                _ => continue,
            }
        }
        None
    }

    // Remove using linear probing and tombstoning (mark as Deleted)
    fn remove(&mut self, key: impl Borrow<str>) -> Option<SValueType> {
        let key = key.borrow();
        let h = fxhash(key);
        let max_attempts = (0.75 * (self.capacity as f64)) as usize;
        for i in 0..max_attempts {
            let bucket_i = (h + i) & (self.capacity - 1);
            match self.buckets[bucket_i] {
                Entry::Occupied(ref k, value, insertion_i) if k == key => {
                    self.size -= 1;

                    self.buckets[bucket_i] = Entry::Deleted; // Set slot as deleted (tomb-stoning)

                    self.by_insertion[insertion_i] = None; // Respective insertion index also pointing nowhere

                    // Now update first/last
                    // If we delete an item which is neither first or last this should be no-op
                    // Let's update last. We can pop items to reuse memory
                    while self.by_insertion.pop_if(|e| e.is_none()).is_some() {}
                    self.last = self.by_insertion.last().map(|index| {
                        let index = index.expect("No trailing Nones");
                        (self.by_insertion.len() - 1, index)
                    });

                    // We might have deleted the first, let's advance (No removing, otherwise insertion_indices invalidate)
                    let cur_first = self.first.expect("Had at least len 1").0;
                    self.first = (cur_first..self.by_insertion.len())
                        .find_map(|i| self.by_insertion[i].map(|bucket| (i, bucket)));

                    return Some(value);
                }
                Entry::Empty => return None,
                _ => continue,
            };
        }
        None
    }

    /// returns the most recent key-value pair that was either inserted or updated and is still present,
    fn get_last(&self) -> Option<(&SKeyType, &SValueType)> {
        self.last
            .map(|(_, bucket_i)| match &self.buckets[bucket_i] {
                Entry::Occupied(key, value, _) => (key, value),
                _ => panic!("Index to deleted entry"),
            })
    }

    /// returns the least recent key-value pair that was either inserted or updated and is still present
    fn get_first(&self) -> Option<(&SKeyType, &SValueType)> {
        self.first
            .map(|(_, bucket_i)| match &self.buckets[bucket_i] {
                Entry::Occupied(key, value, _) => (key, value),
                _ => panic!("Index to deleted entry"),
            })
    }

    fn len(&self) -> usize {
        self.size
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_insert() {
        let mut table = StrHashTable::new(1000);
        assert_eq!(table.get("World"), None);

        table.insert("Hello".into(), 1).unwrap();
        table.insert("World".into(), 2).unwrap();
        table.insert("Fer".into(), 3).unwrap();

        assert_eq!(table.len(), 3);

        assert_eq!(table.get("Hello").unwrap(), &1);
        assert_eq!(table.get("World").unwrap(), &2);
        assert_eq!(table.get("Fer").unwrap(), &3);

        // And remove
        let out = table.remove("World");
        assert_eq!(out, Some(2));
        assert_eq!(table.get("World"), None);
    }

    #[test]
    fn test_base_first_last() {
        let mut table = StrHashTable::new(1000);
        assert_eq!(table.get_first(), None);
        assert_eq!(table.get_last(), None);

        table.insert("Hello".into(), 1).unwrap();
        assert_eq!(table.get_first().unwrap(), (&"Hello".into(), &1));
        assert_eq!(table.get_last().unwrap(), (&"Hello".into(), &1));

        table.insert("World".into(), 2).unwrap();
        assert_eq!(table.get_first().unwrap(), (&"Hello".into(), &1));
        assert_eq!(table.get_last().unwrap(), (&"World".into(), &2));
    }

    #[test]
    fn test_advanced_first_last() {
        let mut table = StrHashTable::new(1000);
        table.insert("Hello".into(), 1).unwrap();
        table.insert("World".into(), 2).unwrap();

        table.remove("Hello");
        assert_eq!(table.get_first().unwrap(), (&"World".into(), &2));
        assert_eq!(table.get_last().unwrap(), (&"World".into(), &2));

        table.remove("World");
        assert_eq!(table.get_first(), None);
        assert_eq!(table.get_last(), None);
    }

    #[test]
    fn test_advanced_last_first() {
        let mut table = StrHashTable::new(1000);
        table.insert("Hello".into(), 1).unwrap();
        table.insert("World".into(), 2).unwrap();

        table.remove("World");
        assert_eq!(table.get_first().unwrap(), (&"Hello".into(), &1));
        assert_eq!(table.get_last().unwrap(), (&"Hello".into(), &1));

        table.remove("Hello");
        assert_eq!(table.get_first(), None);
        assert_eq!(table.get_last(), None);
    }
}
