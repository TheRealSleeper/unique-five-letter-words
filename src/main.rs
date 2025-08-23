use arraystring::{ArrayString, typenum::U5};
use itertools::Itertools;
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    let t_start = Instant::now();

    // Find all valid words and u32 mask where each bit represents a particular letter
    let words: Vec<(ArrayString<U5>, u32)> = std::fs::read_to_string("words_alpha.txt")
        .unwrap()
        .lines()
        .map(|w| w.trim())
        .filter(|w| w.len() == 5)
        .map(|w| {
            (
                arraystring::ArrayString::from(w),
                w.chars().fold(0_u32, |acc, c| acc | (1 << (c as u8 - b'a'))),
            )
        })
        .filter(|w| w.1.count_ones() == 5)
        .sorted_unstable_by(|a, b| a.0.cmp(&b.0))
        .collect_vec();

    // Collect all unique masks found in word set
    let char_masks = words
        .iter()
        .map(|w| w.1)
        .sorted_unstable()
        .dedup()
        .collect_vec();

    // Collect words into groups sharing the same masks
    let mut filtered_words = std::collections::HashMap::new();
    for &mask in char_masks.iter() {
        filtered_words.insert(
            mask,
            words
                .iter()
                .filter_map(|(w, m)| (mask == *m).then_some(w))
                .cloned()
                .collect_vec(),
        );
    }

    #[cfg(debug_assertions)]
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    // Find all mask combinations that solve the challenge
    let combos = char_masks
        .par_iter()
        .enumerate()
        .map(|(i, &mask1)| {
            // println!("At {i} of {}", char_masks.len());
            let mut combos = Vec::with_capacity(10);
            let combo_mask = mask1;
            for (ii, &mask2) in char_masks.iter().enumerate().skip(i + 1) {
                if combo_mask & mask2 == 0 {
                    let combo_mask = combo_mask | mask2;
                    for (iii, &mask3) in char_masks.iter().enumerate().skip(ii + 1) {
                        if combo_mask & mask3 == 0 {
                            let combo_mask = combo_mask | mask3;
                            for (iv, &mask4) in char_masks.iter().enumerate().skip(iii + 1) {
                                if combo_mask & mask4 == 0 {
                                    let combo_mask = combo_mask | mask4;
                                    for &mask5 in char_masks.iter().skip(iv + 1) {
                                        if combo_mask & mask5 == 0 {
                                            combos.push([mask1, mask2, mask3, mask4, mask5]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            #[cfg(debug_assertions)]
            {
                let num_completed = count.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                let percent_complete = num_completed as f32 / char_masks.len() as f32 * 100.0;
                let elapsed = t_start.elapsed();
                let time_remaining = elapsed.as_secs_f32() / num_completed as f32 * (char_masks.len() - num_completed) as f32;
                println!(
                    "Finished {num_completed} of {} masks, {percent_complete:.2}% complete, estimated {time_remaining:.0} seconds remaining",
                    char_masks.len()
                );
            }

            combos
        })
        .flatten()
        .collect::<Vec<_>>();

    // Make every possible word combination from valid mask combinations
    let answer = combos
        .iter()
        .flat_map(|masks| {
            masks
                .iter()
                .map(|mask| filtered_words.get(mask).unwrap())
                .multi_cartesian_product()
                .map(|v| v.into_iter().format(", ").to_string())
                .collect_vec()
        })
        .sorted()
        .format("\n")
        .to_string();

    std::fs::write("answer.txt", &answer).unwrap();
    println!(
        "found {} solutions in {:.3} seconds",
        answer.lines().count(),
        t_start.elapsed().as_secs_f32()
    );
}
