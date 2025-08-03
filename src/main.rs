use itertools::Itertools;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

fn main() {
    let t_start = Instant::now();
    let words = Arc::new(
        std::fs::read_to_string("words_alpha.txt")
            .unwrap()
            .lines()
            .map(|w| w.trim())
            .filter(|w| w.len() == 5)
            .sorted()
            .map(|w| {
                (
                    w.to_string(),
                    w.chars().fold(0_u32, |acc, c| acc | 1 << c as u8 - b'a'),
                )
            })
            .sorted_unstable_by(|a, b| a.0.cmp(&b.0))
            .collect_vec(),
    );

    let char_masks = words
        .iter()
        .map(|w| w.1)
        .sorted_unstable()
        .dedup()
        .collect_vec();
    let mut filtered_words = std::collections::HashMap::new();
    for mask in char_masks.iter() {
        filtered_words.insert(
            *mask,
            words
                .iter()
                .filter_map(|(w, m)| (*mask == *m).then_some(w))
                .cloned()
                .collect_vec(),
        );
    }
    let filtered_words = Arc::new(filtered_words);

    let threads = std::thread::available_parallelism().unwrap().get();
    println!(
        "{} valid words in list, and {threads} threads available",
        words.len()
    );

    let i_shared = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    for _ in 0..threads {
        let filtered_words = filtered_words.clone();
        let i_shared = i_shared.clone();
        let masks = char_masks.clone();

        handles.push(std::thread::spawn(move || {
            let mut sets = Vec::new();
            while i_shared.load(std::sync::atomic::Ordering::Acquire) < filtered_words.len() {
                let mut mask_sets = Vec::new();
                let i = i_shared.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                println!("Checking from {i} of {} masks", filtered_words.len());
                for ii in (i + 1)..masks.len() {
                    if (masks[i] | masks[ii]).count_ones() != 10 {
                        continue;
                    }
                    for iii in (ii + 1)..masks.len() {
                        if (masks[i] | masks[ii] | masks[iii]).count_ones() != 15 {
                            continue;
                        }
                        for iii in (ii + 1)..masks.len() {
                            if (masks[i] | masks[ii] | masks[iii]).count_ones() != 15 {
                                continue;
                            }
                            for iv in (iii + 1)..masks.len() {
                                if (masks[i] | masks[ii] | masks[iii] | masks[iv]).count_ones()
                                    != 20
                                {
                                    continue;
                                }
                                for v in (iv + 1)..masks.len() {
                                    if (masks[i] | masks[ii] | masks[iii] | masks[iv] | masks[v])
                                        .count_ones()
                                        != 25
                                    {
                                        continue;
                                    }
                                    mask_sets.push([
                                        masks[i], masks[ii], masks[iii], masks[iv], masks[v],
                                    ]);
                                }
                            }
                        }
                    }
                }
                
                // for mask in masks[i..masks.len()]
                //     .iter()
                //     .combinations(5)
                //     .filter(|combo| combo.iter().fold(0, |acc, n| acc | **n) == 25)
                // {
                // 
                for mask in mask_sets.into_iter().sorted_unstable().dedup() {
                    for word1 in filtered_words[&mask[0]].iter() {
                        for word2 in filtered_words[&mask[1]].iter() {
                            for word3 in filtered_words[&mask[2]].iter() {
                                for word4 in filtered_words[&mask[3]].iter() {
                                    for word5 in filtered_words[&mask[4]].iter() {
                                        sets.push(format!(
                                            "{word1}, {word2}, {word3}, {word4}, {word5}"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            sets
        }));
    }
    let mut sets = Vec::with_capacity(threads);
    for handle in handles {
        sets.push(handle.join().unwrap());
    }

    let answer = sets.iter().flatten().sorted().format("\n").to_string();
    std::fs::write("answer.txt", &answer).unwrap();
    println!(
        "found {} solutions in {:.3} seconds",
        answer.lines().count(),
        t_start.elapsed().as_secs_f32()
    );
}
