
use compact_str::CompactString;
use core::{borrow::Borrow, hash::Hash};
use fxhash::hash as fxhash;
use alloc::vec::Vec;

use unix_print::unix_println as println;

#[derive(Debug)]
#[repr(u8)]
enum Entry<K: Hash + Eq, V: Copy> {
    Empty,
    Occupied(K, V),
    Deleted,
}

pub struct HashTable<K: Hash + Eq, V: Copy> {
    buckets: Vec<Entry<K, V>>,
    size: usize,
}

type KeyType = CompactString;
type ValueType = u32;
pub type StrHashTable = HashTable<KeyType, ValueType>;

impl StrHashTable {

    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        let mut buckets = Vec::with_capacity(capacity);
        buckets.resize_with(capacity, || Entry::Empty);
        Self { buckets, size: 0}
    }

    pub fn insert(&mut self, key: KeyType, value: ValueType) -> Result<(), &'static str> {
        let h = fxhash(&key);
        let max_attempts = (0.75 * (self.capacity() as f64)) as usize;
        for i in 0..max_attempts {
            let bucket_i = (h+i) % self.capacity();
            if let Entry::Empty = self.buckets[bucket_i] {
                println!( "Adding to position {}", bucket_i);
                self.buckets[bucket_i] = Entry::Occupied(key, value);
                self.size += 1;
                return Ok(());
            }
        }
        return Err("Could not insert. No sufficient slots");
    }

    /// Lookup using linear probing
    pub fn get(&self, key: impl Borrow<str>) -> Option<&ValueType> {
        let key = key.borrow();
        let h = fxhash(key);
        let max_attempts = (0.75 * (self.capacity() as f64)) as usize;
        for i in 0..max_attempts {
            let bucket_i = (h+i) % self.capacity();
            match &self.buckets[bucket_i] {
                Entry::Occupied(k, value) if k == key => {
                    println!( "Found in position {}", bucket_i);
                    return Some(value)
                },
                Entry::Empty => return None,
                _ => continue,
            }
        }
        None
    }

    /// Remove using linear probing and tombstoning (mark as Deleted)
    pub fn remove(&mut self, key: &KeyType) -> Option<ValueType> {
        let h = fxhash(&key);
        let max_attempts = (0.75 * (self.capacity() as f64)) as usize;
        for i in 0..max_attempts {
            let bucket_i = (h+i) % self.capacity();
            match &self.buckets[bucket_i] {
                Entry::Occupied(k, v) if k == key => {
                    let out = *v;
                    self.buckets[bucket_i] = Entry::Deleted;
                    return Some(out);
                }
                Entry::Empty => return None,
                _ => continue,
            };
        }
        None
    }

    /// returns the most recent key-value pair that was either inserted or updated and is still present,
    pub fn get_last(&self, _key: &KeyType) -> Option<&ValueType> {
        None
    }

    /// returns the least recent key-value pair that was either inserted or updated and is still present
    pub fn get_first(&self, _key: &KeyType) -> Option<&ValueType> {
        None
        
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.buckets.len()
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
