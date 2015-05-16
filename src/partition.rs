//! Provides a general partitioning function that implements C++'s
//! [std::partition](http://en.cppreference.com/w/cpp/algorithm/partition)
use std::mem;

/// Re-orders elements in the range yielded by `it` based on `pred`. All elements
/// that the predicate returns true for will be placed before all elements
/// that the predicate returned false for. Also returns the index of the
/// first element in the false group
pub fn partition<'a, T: 'a, I, F>(mut it: I, pred: F) -> usize
        where I: DoubleEndedIterator<Item = &'a mut T>,
        F: Fn(&T) -> bool {
    let mut split_idx = 0;
    loop {
        let mut front = None;
        let mut back = None;
        while let Some(f) = it.next() {
            if !pred(f) {
                front = Some(f);
                break;
            } else {
                split_idx += 1;
            }
        }
        while let Some(b) = it.next_back() {
            if pred(b) {
                back = Some(b);
                break;
            }
        }
        match (front, back) {
            (Some(f), Some(b)) => {
                mem::swap(f, b);
                split_idx += 1;
            },
            _ => break,
        }
    }
    split_idx
}

#[test]
fn test_partition() {
    // This test just partitions the odd and even numbers
    let mut vals = vec![1u32, 2, 3, 4, 5, 6];
    let idx = partition(vals.iter_mut(), |x| *x % 2 == 0);
    println!("Partition idx = {}", idx);
    // The first 3 items should be even the next 3 should be odd
    println!("Partitioned: {:?}", vals);
    assert_eq!(idx, 3);
    assert!(vals.iter().take(3).fold(true, |f, x| *x % 2 == 0 && f));
    assert!(vals.iter().skip(3).fold(true, |f, x| *x % 2 != 0 && f));
}

