mod cache;
use cache::Cache;

fn main() {
    // 1) cache integers: load by parsing, process by doubling
    let mut int_cache = Cache::new(
        |s| s.parse::<i32>().expect("not an int"),
        |n| n * 2,
    );

    let v1 = int_cache.get("21");
    println!("first get(\"21\") = {}", v1);  // 42

    let v2 = int_cache.get("21");
    println!("second get(\"21\") = {}", v2); // 42, from cache

    // now remove twice → counter 2 → 1 → 0 then evicted
    int_cache.remove("21");
    int_cache.remove("21");
    assert!(int_cache.get_cached("21").is_none());

    let v3 = int_cache.get("21");
    println!("after eviction, get(\"21\") = {}", v3); // reload -> 42

    // 2) cache strings: load returns the raw &str -> String, process prepends text
    let mut string_cache = Cache::new(
        |s| s.to_string(),
        |s| format!("processed: {}", s),
    );

    let s1 = string_cache.get("hello");
    println!("{}", s1); // "processed: hello"

    let s2 = string_cache.get_cached("hello").unwrap();
    println!("{}", s2); // again, from cache
}
