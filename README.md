
# HashMap with Open Addressing and Insertion Tracking

This is a fixed-capacity hash table implemented in Rust using **open addressing with linear probing**. Unlike standard hash maps, it maintains insertion order tracking and supports efficient access to the **oldest** and **most recent** active entries.

---

## Problem Description

Traditional hash tables are optimized for speed but often lack insertion history tracking. This limits their utility in scenarios where order matters, such as LRU caches, event tracking, or audit logs.

This implementation solves that by:

- Using **open addressing and linear probing** for collision resolution.
- Maintaining an **insertion vector** to record entry order.
- Tracking both the **first** and **last** valid entries after insertions and deletions.

---

## Data Structure

The data structure consists of two tightly coupled vectors:

### 1. Entry Vector (`buckets`)
This is the core hash map, holding actual key-value pairs.

```rust
enum Entry<K, V> {
    Empty,
    Deleted,
    Occupied(K, V, usize), // usize: index into insertion vector
}
```

- **Empty**: Slot has never been used.
- **Deleted**: Slot was used but is now logically removed (tombstone).
- **Occupied**: Stores `(key, value)` and the index into the insertion order vector.

### 2. Insertion Vector (`by_insertion`)
Preserves the order in which entries were inserted or updated.

```rust
Vec<Option<usize>> // maps insertion index -> bucket index
```

- `Some(bucket_index)`: Entry is still valid.
- `None`: Entry has been deleted.

This allows iteration in order and efficient tracking of "first" and "last" valid entries.

---

### ASCII Diagram

```
 Entry Vector (buckets):
+-------+------------------------------+
| Index | Entry                       |
+-------+------------------------------+
| 0     | Empty                       |
| 1     | Occupied("A", 100, 0)       |  <--- insertion index 0
| 2     | Deleted                     |
| 3     | Occupied("B", 200, 1)       |  <--- insertion index 1
| 4     | Occupied("C", 300, 2)       |  <--- insertion index 2
+-------+------------------------------+

 Insertion Vector (by_insertion):
+------------+-------------------+
| Insertion  | Bucket Index      |
+------------+-------------------+
| 0          | Some(1)           |  --> "A"
| 1          | Some(3)           |  --> "B"
| 2          | Some(4)           |  --> "C"
+------------+-------------------+

 After removing "B":
 Entry at 3 → Deleted
 Insertion index 1 → None

 Insertion Vector:
+------------+-------------------+
| 0          | Some(1)           |
| 1          | None              |  <--- removed
| 2          | Some(4)           |
+------------+-------------------+
```

---

## Algorithm

### Insert

1. Hash the key and use linear probing to find an `Empty` or `Deleted` slot.
2. Store `(key, value, insertion_index)` in the entry vector.
3. Push `Some(bucket_index)` to the insertion vector.
4. Update `first` and `last` if needed.

### Get

1. Hash the key and probe linearly.
2. Return value if a matching `Occupied` entry is found.

### Remove

1. Hash and probe to find the entry.
2. Mark it as `Deleted`.
3. Set the corresponding insertion vector entry to `None`.
4. Adjust `first` and `last`:
   - `first`: Advance to the next non-`None` item.
   - `last`: Trim trailing `None` entries.

### Track First and Last

"first" and "last" are adjusted as required by `insert` and `remove` functions. They hold a tuple containing:
 - the index of the entry in the `by_insertion` vector (for control), as well as the index in the entry vector (for direct data access)
`get_first()` and `get_last()`, using the index, will retrieve the entry data.

---

## Notes

- Hash function: `fxhash`, a fast, non-cryptographic hash.
- Capacity is rounded up to the next power of two.
- Load factor is limited to ~75% to control probing cost.
- Safe updates of insertion metadata avoid unsafe references.
