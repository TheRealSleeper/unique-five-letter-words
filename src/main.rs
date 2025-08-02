use arraystring::ArrayString;
use arraystring::typenum::U5;
use itertools::Itertools;
use std::collections::HashSet;
use std::ops::Add;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::Instant;

fn main() {
    let t_start = Instant::now();
    let words = std::fs::read_to_string("words.txt")
        .unwrap()
        .lines()
        .map(|w| w.to_ascii_lowercase())
        .filter(|w| w.len() == 5)
        .filter(|w| {
            w.chars().sorted_unstable().dedup().count() == 5
                && w.chars().all(|c| c.is_ascii_alphabetic())
        })
        .map(|w| ArrayString::from(w.as_str()))
        .collect::<Vec<ArrayString<U5>>>();
    println!("{} valid words in list", words.len());

    // for word in words.iter() {
    //     println!("{word}");
    // }
    let combos: Arc<RwLock<HashSet<[Option<usize>; 5]>>> =
        Arc::new(RwLock::new(HashSet::with_capacity(words.len() / 5)));
    let threads = std::thread::available_parallelism().unwrap().get();
    // println!("{threads} threads available");
    let i_shared = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    // let output: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));

    let mut handles = Vec::with_capacity(threads);
    for _ in 0..threads {
        let combos = combos.clone();
        // let output = output.clone();
        let words = words.clone();
        let i_shared = i_shared.clone();

        handles.push(std::thread::spawn(move || {
            while i_shared.load(Ordering::Acquire) < words.len() {
                let i = i_shared.fetch_add(1, Ordering::AcqRel);
                if i >= words.len() {
                    break;
                }
                println!(
                    "At word {i}, {:.3}% complete, {:.1} seconds remaining",
                    i.saturating_sub(threads) as f32 / words.len() as f32 * 100.0,
                    t_start.elapsed().as_secs_f32()
                        / (i.saturating_sub(threads - 1) as f32 / words.len() as f32)
                        * (1.0 - i as f32 / words.len() as f32)
                );

                let found_words = [None; 5];
                get_words(0, found_words, combos.clone(), &words, 0, i);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    for set in combos.read().unwrap().iter() {
        println!(
            "{}, {}, {}, {}, {}",
            words[set[0].unwrap()],
            words[set[1].unwrap()],
            words[set[2].unwrap()],
            words[set[3].unwrap()],
            words[set[4].unwrap()]
        );
    }

    // println!("{}", output.read().unwrap());

    println!(
        "Found {} sets of five letter words made up of 25 unique letters in {:.3} seconds",
        combos.read().unwrap().len(),
        t_start.elapsed().as_secs_f64()
    );
}

trait IndexBits {
    fn set_bit(&mut self, n: u8, val: bool) -> Self;
    fn get_bit(self, n: u8) -> bool;
}

impl IndexBits for u32 {
    #[inline]
    fn set_bit(&mut self, n: u8, val: bool) -> Self {
        if val {
            *self |= 1 << n;
        } else {
            *self &= !(1 << n);
        }

        *self
    }

    #[inline]
    fn get_bit(self, n: u8) -> bool {
        (self >> n & 1) == 1
    }
}

fn get_words(
    found_chars: u32,
    found_words: [Option<usize>; 5],
    combos: Arc<RwLock<HashSet<[Option<usize>; 5]>>>,
    words: &Vec<ArrayString<U5>>,
    depth: usize,
    word_position: usize,
) {
    if words[word_position]
        .chars()
        .all(|c| !found_chars.get_bit(c as u8 - b'a'))
    {
        let mut found_words = found_words;
        found_words[depth] = Some(word_position);

        let mut found_chars = found_chars;
        for c in words[word_position].bytes() {
            found_chars.set_bit(c - b'a', true);
        }

        if depth < 4 {
            for ii in word_position.add(1)..words.len() {
                get_words(
                    found_chars,
                    found_words,
                    combos.clone(),
                    words,
                    depth + 1,
                    ii,
                );
            }
        } else {
            found_words.sort();
            println!(
                "{}, {}, {}, {}, {}",
                words[found_words[0].unwrap()],
                words[found_words[1].unwrap()],
                words[found_words[2].unwrap()],
                words[found_words[3].unwrap()],
                words[found_words[4].unwrap()]
            );
            combos.write().unwrap().insert(found_words);
        }
    }
}

#[test]
fn index_int_test() {
    assert_eq!(3, 1.set_bit(1, true));
    assert_eq!(8, 0.set_bit(3, true));
    assert_eq!(0, 1.set_bit(0, false));
    assert_eq!(1, 3.set_bit(1, false));
    assert_eq!(true, 1.get_bit(0));
    assert_eq!(false, 1.get_bit(1));
}
