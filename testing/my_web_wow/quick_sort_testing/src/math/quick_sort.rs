use std::cmp::Ordering;

/// Swap two elements in `items`.
pub fn swap_items<T>(items: &mut [T], i: usize, j: usize) {
    items.swap(i, j);
}

/// Partition `items[left..=right]` around a pivot, returning the next split index.
/// Requires `T: Clone` so we can clone the pivot out and avoid holding a borrow across swaps.
pub fn partition<T: Clone>(
    items: &mut [T],
    left: usize,
    right: usize,
    cmp: &dyn Fn(&T, &T) -> Ordering,
) -> usize {
    let pivot_index = (left + right) / 2;
    // clone pivot so we don’t keep a borrow into `items`
    let pivot = items[pivot_index].clone();
    let mut i = left;
    let mut j = right;

    while i <= j {
        while cmp(&items[i], &pivot) == Ordering::Less {
            i += 1;
        }
        while cmp(&items[j], &pivot) == Ordering::Greater {
            // safely underflow‐aware
            if j == 0 { break; }
            j -= 1;
        }
        if i <= j {
            items.swap(i, j);
            i += 1;
            if j == 0 { break; }
            j -= 1;
        }
    }

    i
}

/// Recursively quick-sort `items[left..=right]` with the given `cmp`.
pub fn quick_sort_array<T: Clone>(
    items: &mut [T],
    left: usize,
    right: usize,
    cmp: &dyn Fn(&T, &T) -> Ordering,
) {
    if left < right {
        let idx = partition(items, left, right, cmp);
        if idx > 0 && left < idx - 1 {
            quick_sort_array(items, left, idx - 1, cmp);
        }
        if idx < right {
            quick_sort_array(items, idx, right, cmp);
        }
    }
}

/// Multi-key quick-sort:
/// 1) primary sort by `comparators[0]`
/// 2) for each subsequent comparator, re-sort each tied group under all earlier keys
pub fn multi_quick_sort<T: Clone>(
    items: &mut [T],
    left: usize,
    right: usize,
    comparators: &[&dyn Fn(&T, &T) -> Ordering],
) {
    if comparators.is_empty() {
        panic!("At least one comparator required");
    }

    // 1) primary
    quick_sort_array(items, left, right, comparators[0]);

    // 2) refine ties
    for c in 1..comparators.len() {
        let cmp = comparators[c];
        let mut group_start = left;
        let mut i = left + 1;
        while i <= right {
            // see if items[group_start] vs items[i] differ under any earlier comparator
            let mut different = false;
            for prev in 0..c {
                if (comparators[prev])(&items[group_start], &items[i]) != Ordering::Equal {
                    different = true;
                    break;
                }
            }
            if different {
                // we have a tied group [group_start .. i-1]
                if i > group_start + 1 {
                    quick_sort_array(items, group_start, i - 1, cmp);
                }
                group_start = i;
            }
            i += 1;
        }
        // last pending group
        if i > group_start + 1 {
            quick_sort_array(items, group_start, i - 1, cmp);
        }
    }
}
