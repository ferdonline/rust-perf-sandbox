use alloc::vec::Vec;
use compact_str::CompactString;
use core::{borrow::Borrow, hash::Hash};
use fxhash::hash as fxhash;

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
    fn remove(&mut self, key: &K) -> Option<V>;

    /// Returns the most recent key-value pair that was either inserted or updated and is still present
    fn get_last(&self, _key: &K) -> Option<&V>;

    /// Returns the least recent key-value pair that was either inserted or updated and is still present
    fn get_first(&self, _key: &K) -> Option<&V>;

    /// Returns the number of map entries
    fn len(&self) -> usize;
}

#[derive(Debug)]
#[repr(u8)]
enum Entry<K, V> {
    Empty,
    Occupied(K, V),
    Deleted,
}

type SKeyType = CompactString;
type SValueType = u32;

#[derive(Debug)]
pub struct StrHashTable {
    // The map entries
    // @note: Because we need to track latest and first the buckets don't hold the Values
    // but the index of the element as per insertion order (next field)
    // Keeping an index is a simple way in rust to keep a reference without using cells and
    // other Rust structures which would incur a runtime penalty.
    // Even if the values are a separate structure and we incur one indirection, the Key is still
    // inlined (100%, thanks to CompactString) and so searching for the item should still benefit
    // from cache locality
    buckets: Vec<Entry<SKeyType, usize>>,
    // The Values of the map per insertion order. Deleted items are set to None, so indices are
    // kept valid and we don't incur the traditional runtime penalty of really removing the items
    elements: Vec<Option<SValueType>>,
    // the map capacity (cache)
    capacity: usize,
    // Number of map entries
    size: usize,
    // Index of the first inserted still valid
    _first: Option<usize>,
    // Index of the last inserted still valid
    _last: Option<usize>,
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
            elements: Vec::new(),
            _first: None,
            _last: None,
        }
    }

    fn insert(&mut self, key: SKeyType, value: SValueType) -> Result<(), &'static str> {
        let h = fxhash(&key);
        let max_attempts = (0.75 * (self.capacity as f64)) as usize;
        for i in 0..max_attempts {
            let bucket_i = (h + i) & (self.capacity - 1);
            if let Entry::Empty = self.buckets[bucket_i] {
                println!("Adding to position {}", bucket_i);
                self.elements.push(Some(value));
                self.buckets[bucket_i] = Entry::Occupied(key, self.elements.len() - 1);
                self.size += 1;
                return Ok(());
            }
        }
        return Err("Could not insert. No sufficient slots");
    }

    // Lookup using linear probing
    fn get(&self, key: impl Borrow<str>) -> Option<&SValueType> {
        let key = key.borrow();
        let h = fxhash(key);
        let max_attempts = (0.75 * (self.capacity as f64)) as usize;
        for i in 0..max_attempts {
            let bucket_i = (h + i) & (self.capacity - 1);
            match &self.buckets[bucket_i] {
                Entry::Occupied(k, value) if k == key => {
                    println!("Found in position {}", bucket_i);
                    return self.elements[*value].as_ref();
                }
                Entry::Empty => return None,
                _ => continue,
            }
        }
        None
    }

    // Remove using linear probing and tombstoning (mark as Deleted)
    fn remove(&mut self, key: &SKeyType) -> Option<SValueType> {
        let h = fxhash(&key);
        let max_attempts = (0.75 * (self.capacity as f64)) as usize;
        for i in 0..max_attempts {
            let bucket_i = (h + i) & (self.capacity - 1);
            match &self.buckets[bucket_i] {
                Entry::Occupied(k, v) if k == key => {
                    // Copy the index
                    let element_i = *v;
                    // Set slot as deleted (tombstoning)
                    self.buckets[bucket_i] = Entry::Deleted;
                    // Take the actual value
                    let value = self.elements[element_i].take();
                    // Reduce count
                    self.size -= 1;
                    return value;
                }
                Entry::Empty => return None,
                _ => continue,
            };
        }
        None
    }

    /// returns the most recent key-value pair that was either inserted or updated and is still present,
    fn get_last(&self, _key: &SKeyType) -> Option<&SValueType> {
        None
    }

    /// returns the least recent key-value pair that was either inserted or updated and is still present
    fn get_first(&self, _key: &SKeyType) -> Option<&SValueType> {
        None
    }

    fn len(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x() {
        let mut table = StrHashTable::new(1000);

        table.insert("Hello".into(), 1).unwrap();
        table.insert("World".into(), 2).unwrap();
        table.insert("Fer".into(), 3).unwrap();

        assert_eq!(table.len(), 3);

        assert_eq!(table.get("Hello").unwrap(), &1);
        assert_eq!(table.get("World").unwrap(), &2);
        assert_eq!(table.get("Fer").unwrap(), &3);
    }
}
