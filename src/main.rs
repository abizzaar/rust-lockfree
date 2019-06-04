// see blog post of arr_macro here: https://www.joshmcguigan.com/blog/array-initialization-rust/
use arr_macro::arr;
use std::sync::atomic::{AtomicI32, Ordering};
fn main() {}

struct Entry {
    pub key: AtomicI32,
    pub value: AtomicI32,
}

struct HashTable {
    pub entries: [Entry; 1000],
}

impl HashTable {
    pub fn new() -> Self {
        HashTable {
            entries: arr![Entry {key: AtomicI32::new(0), value: AtomicI32::new(0)}; 1000],
        }
    }

    pub fn set_item(&mut self, key: i32, val: i32) {
        for e in self.entries.iter() {
            let cas_result = e.key.compare_and_swap(0, key, Ordering::SeqCst);
            if cas_result == 0 || cas_result == key {
                e.value.store(val, Ordering::SeqCst);
                return;
            }
        }
    }

    pub fn get_item(&self, key: i32) -> i32 {
        for e in self.entries.iter() {
            let load_result = e.key.load(Ordering::SeqCst);
            // println!("on {} {}", load_result, e.value.load(Ordering::SeqCst));
            if load_result == key {
                return e.value.load(Ordering::SeqCst);
            } else if load_result == 0 {
                return 0;
            }
        }

        0
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use rand::{random, Rng};
    use std::cell::UnsafeCell;
    use std::marker::{Send, Sync};
    use std::sync::Arc;
    use std::{thread, time};

    #[test]
    fn sequential() {
        let mut h = HashTable::new();
        h.set_item(1, 1);
        assert_eq!(1, h.get_item(1));
        h.set_item(2, 2);
        h.set_item(3, 33);
        assert_eq!(1, h.get_item(1));
        assert_eq!(2, h.get_item(2));
        assert_eq!(33, h.get_item(3));
    }

    #[test]
    fn multithreaded_simple() {
        // Need to use UnsafeCell b/c, by default, Arc cannot be passed mutably across threads

        // Need to make new struct b/c cannot impl Sync/Send for UnsafeCell directly
        struct NotThreadSafe<T> {
            value: UnsafeCell<T>,
        }
        unsafe impl<T> Sync for NotThreadSafe<T> {}
        unsafe impl<T> Send for NotThreadSafe<T> {}

        let h = HashTable::new();
        let arc_h = Arc::new(NotThreadSafe {
            value: UnsafeCell::new(h),
        });

        for i in 1..1000 {
            let arc_h1 = arc_h.clone();
            unsafe {
                thread::spawn(move || (*arc_h1.value.get()).set_item(i, i));
            }

            let arc_h2 = arc_h.clone();
            unsafe {
                thread::spawn(move || {
                    let millis = generate_random_duration();
                    thread::sleep(millis);
                    let val = (*arc_h2.value.get()).get_item(i);
                    assert!(i == val || 0 == val);
                });
            }
        }
    }

    fn generate_random_duration() -> time::Duration {
        let mut rng = rand::thread_rng();
        let rand: u64 = rng.gen_range(1, 20);
        time::Duration::from_millis(rand)
    }
}
