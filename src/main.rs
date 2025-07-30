use itertools::Itertools;
use rayon::iter::ParallelIterator;
use rayon::str::ParallelString;
use std::collections::HashSet;
use std::ops::Add;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::Instant;

fn main() {
    let t_start = Instant::now();
    let words: Arc<Vec<String>> = Arc::from(
        std::fs::read_to_string("wordle_words.txt")
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
    println!("{} valid words in list", words.len());

    // for word in words.iter() {
    //     println!("{word}");
    // }
    let combos: Arc<RwLock<HashSet<Vec<String>>>> =
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
                println!("At word {i}, {:.3}% complete, {:.1} seconds remaining", 
                    i as f32 / words.len() as f32 * 100.0, 
                    1.0 / (i as f32 / words.len() as f32) * t_start.elapsed().as_secs_f32());

                let found_chars = [false; 26];
                let found_words = Vec::with_capacity(5);
                get_words(found_chars, found_words, combos.clone(), words.clone(), 0, i);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    for set in combos.read().unwrap().iter() {
        println!("{}, {}, {}, {}, {}", set[0], set[1], set[2], set[3], set[4]);
    }

    // println!("{}", output.read().unwrap());

    println!(
        "Found {} sets of five letter words made up of 25 unique letters in {:.3} seconds",
        combos.read().unwrap().len(),
        t_start.elapsed().as_secs_f64()
    );
}

// /// Checks if the characters in a given word have been used in another word.
// /// If not, it adds the characters of the word to the found character array
// fn check_chars(found_chars: &mut [bool; 26], word: &str) -> bool {
//     // print!("Checking {}: ", word);
//     if word
//         .chars()
//         .all(|c| !found_chars[(c as u8 - b'a') as usize])
//     {
//         for c in word.bytes() {
//             found_chars[(c - b'a') as usize] = true;
//         }
//         // print!("{word} passed!\n");
//         // println!(
//         //     "Letters '{}' still available",
//         //     found_chars
//         //         .iter()
//         //         .enumerate()
//         //         .filter(|(_, c)| !**c)
//         //         .map(|(i, _)| (i as u8 + 'a' as u8) as char)
//         //         .collect::<String>()
//         // );
//         true
//     } else {
//         // print!("Failed!\n");
//         false
//     }
// }

fn get_words(
    found_chars: [bool; 26],
    found_words: Vec<String>,
    combos: Arc<RwLock<HashSet<Vec<String>>>>,
    words: Arc<Vec<String>>, 
    iteration: usize,
    word_position: usize
) {
    if words[word_position]
        .chars()
        .all(|c| !found_chars[(c as u8 - b'a') as usize])
    {
        let mut found_words = found_words.clone(); 
        found_words.push(words[word_position].clone());
        
        let mut found_chars = found_chars;
        for c in words[word_position].bytes() {
            found_chars[(c - b'a') as usize] = true;
        }
        
        if iteration < 5 {
            for ii in word_position.add(1)..words.len() {
                get_words(found_chars, found_words.clone(), combos.clone(), words.clone(), iteration + 1, ii);
            }
        } else {
            found_words.sort(); 
            println!("{}, {}, {}, {}, {}", 
                found_words[0], 
                found_words[1], 
                found_words[2], 
                found_words[3], 
                found_words[4]);
            combos.write().unwrap().insert(found_words);
        }
    } 
}
