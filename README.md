### Rust Performance Challenge 

This README contains explanations for two projects. First, a HashMap implementation, then the Trading Algorithm.

# 1. HashMap with Open Addressing and Insertion Tracking

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

## How to run

Upon compiling, an index_tale binary is created, which should be run from the root of the dir where the file 98-0.txt resides.
Alternatively use cargo run with adequate parameters, e.g.
```sh
$ cargo run --release --bin index_tale
```

The output should look like
```
     Running `target/release/index_tale`
Text contains 138965 unique words

Example of few frequencies:
The: Found 586 times!
lazy: Not found!
fox: Not found!
jumps: Not found!
over: Found 147 times!
the: Found 7524 times!
fence: Not found!

First word and freq are ("\u{feff}The", 1)
Last word and freq are ("eBooks.", 1)
```

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
 - the index of the entry in the `by_insertion` vector (for control)
 - the index in the entry vector (for direct data access). `get_first()` and `get_last()`, using this index, will directly retrieve the entry Key and Value.

---

## Notes

- Hash function: `fxhash`, a fast, non-cryptographic hash.
- Capacity is rounded up to the next power of two.
- Load factor is limited to ~75% to control probing cost.
- Safe updates of insertion metadata avoid unsafe references.


# 2. Trading Specific Algorithms, Parsing

A second program in the crate (api_reader) was implemented to access `binance.com/eapi/v1/ticker` and parse the resulting json.

### How to run

Upon compiling, an api_reader binary is created which can be run from any location.
Alternatively use cargo run with adequate parameters, e.g.
```sh
$ cargo run --release --bin api_reader
```

The output should look like
```
     Running `target/release/api_reader`
"Parsing json" took 4.43ms
Received 1516 data points
```

### Implementation

The main program uses ureq to do the call and obtain the resulting json string.

On a second step we do the parsing. For that end we used serde and serde_json, two ubiquitous crates for parsing and deserializing.
One declared the `Ticker` struct with the expected fields, annotating the serde type and potential required conversions.

Notably, a great deal of the floating point fields come back as a string, therefore requiring parsing to float. `serde` can't do the job alone since e.g. "1.0" is not a json number, but a string, and therefore a conversion parser is required. If not provided serde would complain the types mismatch.

#### A generic parser

Strangely, a generic parser doesn't seem to be provided by `serde` nor readily available online. By creating a generic wrapper function over the native `core::str::parse` one can, in a simple way, brigde this gap and provide a custom deserializer to serde which also applies the parsing, which is based on rock-solid and super optimized built-in `str::parse`.

```rust
fn parse_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<T>().map_err(serde::de::Error::custom)
}
```

In those field which need the conversion through str::parse (into any type) we can simply annotate, e.g.

```rust
struct Ticker {
    #[serde(deserialize_with = "parse_str")]
    priceChange: f32,
 }
```

### Benchmarking

In the main program a sub module `bench` was implemented that benchmarks this parser using serde and serde_json.

To run please execute

```sh
cargo bench --bin api_reader -- bench::bench_deser_ticker
```

The output should be a pleasing 
```
test bench::bench_deser_ticker ... bench:         757.46 ns/iter (+/- 52.72)
```

Demonstrating that we can fully parse one such full structure in ~750 ns.

#### The inner details

Parsing a json structure in ~ 750 ns seems pretty nice, but everything is relative.

Serde is a general framework to serialize and deserialize data, without reflection, based on the Rust's trait system to generate deserialization code which is optimized. Even though it's pretty good, its performance may lack behind certain purpose specific implementations.

Some online sources explain some specific implementations and attempt at improving serde-json which are a good read.

E.g. in https://users.rust-lang.org/t/blog-post-making-slow-rust-code-fast/66074/8 
 - mentions to https://github.com/simd-lite/simd-json (an implementation which makes use of CPU SIMD instructions)
 - points to a specific implementation (using bson) which can be up to 5x faster

A great article which explored and improved performance critical parts of serde_json is given in https://purplesyringa.moe/blog/i-sped-up-serde-json-strings-by-20-percent/


#### Conclusion
For the purpose of performance critical code such optimizations might be worth it, but they come at a cost of maintainability.

Rust promotes collaborative development through shared crates and `simd-json` might be an interesting drop-in replacement for serde_json, attempting to make use of recent CPU architectures.
