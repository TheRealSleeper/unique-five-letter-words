use itertools::Itertools;
use rayon::iter::ParallelIterator;
use rayon::str::ParallelString;
use std::collections::HashSet;
use std::ops::{Add, AddAssign};
use std::sync::{Arc, RwLock};
use std::time::Instant;

fn main() {
    let t_start = Instant::now();
    let words: Arc<Vec<String>> = Arc::from(
        std::fs::read_to_string("words.txt")
            .unwrap()
            .par_lines()
            .map(|w| w.to_ascii_lowercase())
            .filter(|w| w.len() == 5)
            .filter(|w| {
                w.chars().sorted_unstable().dedup().count() == 5
                    && w.chars().all(|c| c.is_ascii_alphabetic())
            })
            .collect::<Vec<String>>(),
    );

    // for word in words.iter() {
    //     println!("{word}");
    // }
    let combos: Arc<RwLock<HashSet<Vec<String>>>> =
        Arc::new(RwLock::new(HashSet::with_capacity(words.len() / 5)));
    let threads = 1; // std::thread::available_parallelism().unwrap().get();
    // println!("{threads} threads available");
    let i_shared: Arc<RwLock<usize>> = Arc::new(RwLock::new(0));

    let mut handles = Vec::with_capacity(threads);
    for _ in 0..threads {
        let combos = combos.clone();
        let words = words.clone();
        let i_shared = i_shared.clone();

        handles.push(std::thread::spawn(move || {
            let mut i_mut = i_shared.write().unwrap();
            while *i_mut < words.len() {
                let i = *i_mut;
                i_mut.add_assign(1);

                let mut found_chars = [false; 256];
                let mut found_words = HashSet::with_capacity(5);

                if check_chars(&mut found_chars, &words[i]) {
                    found_words.insert(&words[i]);
                    for ii in i.add(&1)..words.len() {
                        if check_chars(&mut found_chars, &words[ii]) {
                            found_words.insert(&words[ii]);
                            for iii in ii.add(&1)..words.len() {
                                if check_chars(&mut found_chars, &words[iii]) {
                                    found_words.insert(&words[iii]);
                                    for iv in iii.add(&1)..words.len() {
                                        if check_chars(&mut found_chars, &words[iv]) {
                                            found_words.insert(&words[iv]);
                                            for v in iv.add(&1)..words.len() {
                                                if check_chars(&mut found_chars, &words[v]) {
                                                    if v > 10 {
                                                        panic!();
                                                    }
                                                    found_words.insert(&words[v]);
                                                    combos.write().unwrap().insert(
                                                        found_words
                                                            .iter()
                                                            .sorted()
                                                            .map(|s| (*s).clone())
                                                            .collect(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // for set in combos.read().unwrap().iter() {
    //     println!("{}, {}, {}, {}, {}", set[0], set[1], set[2], set[3], set[4]);
    // }

    println!(
        "Found {} sets of five letter words made up of 25 unique letters in {} seconds",
        combos.read().unwrap().len(),
        t_start.elapsed().as_secs_f64()
    );
}

fn check_chars(found_chars: &mut [bool; 256], word: &str) -> bool {
    print!("Checking {}: ", word);
    let res = word.bytes().all(|b| !found_chars[b as usize]);
    if res {
        for c in word.bytes() {
            found_chars[c as usize] = true
        }
        print!("Passed!\n");
        true
    } else {
        print!("Failed!\n");
        false
    }
}
