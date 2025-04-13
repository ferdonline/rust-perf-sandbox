use std::fs::File;
use std::io::{BufRead, BufReader};

use performance_rust::fixed_hash_table::{HashTable, StrHashTable};

fn main() {
    let reader = BufReader::new(File::open("98-0.txt").expect("Cannot open file 98-0.txt"));

    let mut map = StrHashTable::new(2_000_000);

    for line in reader.lines() {
        for word in line.unwrap().split_whitespace() {
            match map.get(word) {
                None => map.insert(word.into(), 1).unwrap(),
                Some(count) => map.insert(word.into(), count + 1).unwrap(),
            }
        }
    }

    println!("Text contains {} unique words", map.len());

    println!("\nExample of few frequencies:");
    for word in ["The", "lazy", "fox", "jumps", "over", "the", "fence"] {
        match map.get(word) {
            None => println!("{}: Not found!", word),
            Some(count) => println!("{}: Found {} times!", word, count),
        }
    }

    println!("\nFirst word and freq are {:?}", map.get_first().unwrap());
    println!("Last word and freq are {:?}", map.get_last().unwrap());
}
