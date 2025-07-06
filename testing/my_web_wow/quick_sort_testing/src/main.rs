use std::cmp::Ordering;

mod math;
use crate::math::quick_sort::{multi_quick_sort, quick_sort_array};

#[derive(Clone, Debug)]
struct Person {
    name: String,
    age: u32,
    height: f32,
}

fn main() {
    // Sort integers
    let mut numbers = vec![5, 3, 8, 1, 2];
    println!("Before sorting: {:?}", numbers);
    let len = numbers.len();
    if len > 0 {
        quick_sort_array(&mut numbers, 0, len - 1, &|a, b| a.cmp(b));
    }
    println!("After sorting:  {:?}", numbers);
    println!();

    // Sort strings
    let mut words = vec!["banana", "apple", "cherry", "date"];
    println!("Before sorting: {:?}", words);
    let len = words.len();
    if len > 0 {
        quick_sort_array(&mut words, 0, len - 1, &|a, b| a.cmp(b));
    }
    println!("After sorting:  {:?}", words);
    println!();

    // Sort people: primary by age, secondary by height
    let mut people = vec![
        Person { name: "Alice".into(), age: 30, height: 5.5 },
        Person { name: "Bob".into(), age: 25, height: 6.0 },
        Person { name: "Charlie".into(), age: 30, height: 5.8 },
        Person { name: "Diana".into(), age: 25, height: 5.4 },
    ];
    println!("Before sorting:");
    for p in &people {
        println!("{:?}", p);
    }

    let cmp_age = |a: &Person, b: &Person| a.age.cmp(&b.age);
    let cmp_height = |a: &Person, b: &Person| {
        a.height.partial_cmp(&b.height).unwrap_or(Ordering::Equal)
    };

    let len = people.len();
    if len > 0 {
        multi_quick_sort(&mut people, 0, len - 1, &[&cmp_age, &cmp_height]);
    }

    println!("\nAfter sorting by age, then height:");
    for p in &people {
        println!("{:?}", p);
    }
}
