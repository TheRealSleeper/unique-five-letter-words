use anyhow::Result;
use arraystring::{ArrayString, typenum::U5};
use itertools::Itertools;
use rayon::prelude::*;
use std::time::Instant;

fn main() -> Result<()> {
    let t_start = Instant::now();
    let words: Vec<(ArrayString<U5>, u32)> = std::fs::read_to_string("words_alpha.txt")
        .unwrap()
        .lines()
        .map(|w| w.trim())
        .filter(|w| w.chars().sorted().dedup().count() == 5)
        .map(|w| {
            (
                arraystring::ArrayString::from(w),
                w.chars().fold(0_u32, |acc, c| acc | 1 << c as u8 - b'a'),
            )
        })
        .sorted_unstable_by(|a, b| a.0.cmp(&b.0))
        .collect_vec();

    let char_masks = words
        .iter()
        .map(|w| w.1)
        .sorted_unstable()
        .dedup()
        .collect_vec();

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

    // let answers =
    //     char_masks
    //         .par_iter()
    //         .enumerate()
    //         .map(|(i, &m)| {
    //             let mut masks2 = Vec::with_capacity(char_masks.len());
    //             let mut masks3 = Vec::with_capacity(char_masks.len());
    //             let mut masks4 = Vec::with_capacity(char_masks.len());

    //             let filter =
    //                 |combined_mask: u32,
    //                  current_mask: u32,
    //                  previous_mask: u32,
    //                  masks: &[u32],
    //                  filtered_masks: &mut Vec<u32>| {
    //                     filtered_masks.clear();
    //                     filtered_masks.extend(masks.iter().copied().filter(|&mask| {
    //                         combined_mask & mask == 0 && current_mask > previous_mask
    //                     }));
    //                 };

    //             let mut solutions = Vec::new();

    //             println!("{i}");

    //             for &mask1 in char_masks[i + 1..].iter() {
    //                 filter(mask1, mask1, 0, &char_masks, &mut masks2);
    //                 for &mask2 in masks2.iter() {
    //                     let combined_mask = mask1 | mask2;
    //                     filter(combined_mask, mask2, mask1, &char_masks, &mut masks3);
    //                     for &mask3 in masks3.iter() {
    //                         let combined_mask = combined_mask | mask3;
    //                         filter(combined_mask, mask3, mask2, &char_masks, &mut masks4);
    //                         for &mask4 in masks4.iter() {
    //                             if mask4 & combined_mask == 0 && mask4 > mask4 {
    //                                 // println!("Found masks");
    //                                 solutions.push([m, mask1, mask2, mask3, mask4]);
    //                             }
    //                         }
    //                     }
    //                 }
    //             }

    //             solutions
    //         })
    //         .flatten()
    //         .collect::<Vec<[u32; 5]>>();

    // let combo_count = char_masks.iter().combinations(5).count();

    let combos = char_masks
        .par_iter()
        .enumerate()
        .map(|(i, &mask1)| {
            println!("At {i} of {}", char_masks.len());
            let mut combos = Vec::with_capacity(20);
            let combo_mask = mask1;
            for (ii, &mask2) in char_masks[(i + 1)..].iter().enumerate() {
                if combo_mask & mask2 == 0 {
                    let combo_mask = combo_mask | mask2;
                    for (iii, &mask3) in char_masks[(ii + 1)..].iter().enumerate() {
                        if combo_mask & mask3 == 0 {
                            let combo_mask = combo_mask | mask3;
                            for (iv, &mask4) in char_masks[(iii + 1)..].iter().enumerate() {
                                if combo_mask & mask4 == 0 {
                                    let combo_mask = combo_mask | mask4;
                                    for &mask5 in char_masks[(iv + 1)..].iter() {
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
            combos
        })
        .flatten()
        .collect::<Vec<_>>();

    // let combos: Vec<[u32; 5]> = char_masks
    //         .par_iter()
    //         .flat_map(|&m1| {
    //             let mask1 = m1;
    //             let mut uniq_masks2 = Vec::with_capacity(char_masks.len());
    //             let mut uniq_masks3 = Vec::with_capacity(char_masks.len());
    //             let mut uniq_masks4 = Vec::with_capacity(char_masks.len());

    //             let filter = |mask: u32, m: u32, uniq: &[u32], filtered: &mut Vec<u32>| {
    //                 filtered.clear();
    //                 filtered.extend(
    //                     uniq
    //                         .iter()
    //                         .copied()
    //                         .take_while(move |m2| *m2 < m) // Strictly descending to avoid permutations.
    //                         .filter(move |m2| m2 & mask == 0) // Empty intersection to avoid duplicate letters.
    //                 );
    //             };

    //             let mut solutions = Vec::new();
    //             filter(mask1, m1, &char_masks, &mut uniq_masks2);
    //             for &m2 in &uniq_masks2 {
    //                 let mask2 = mask1 | m2;
    //                 filter(mask2, m2, &uniq_masks2, &mut uniq_masks3);
    //                 for &m3 in &uniq_masks3 {
    //                     let mask3 = mask2 | m3;
    //                     filter(mask3, m3, &uniq_masks3, &mut uniq_masks4);
    //                     for &m4 in &uniq_masks4 {
    //                         let mask4 = mask3 | m4;
    //                         for &m5 in &uniq_masks4 {
    //                             if m5 > m4 {
    //                                 break;
    //                             }

    //                             if m5 & mask4 == 0 {
    //                                 solutions.push([m1, m2, m3, m4, m5]);
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             solutions
    //         })
    //         .collect();

    let answer = combos
        .iter()
        .map(|masks| {
            masks
                .iter()
                .map(|mask| filtered_words.get(mask).unwrap())
                .multi_cartesian_product()
                .map(|v| v.into_iter().format(", "))
                .collect_vec()
        })
        .flatten()
        .format("\n")
        .to_string();

    std::fs::write("answer.txt", &answer).unwrap();
    println!(
        "found {} solutions in {:.3} seconds",
        answer.lines().count(),
        t_start.elapsed().as_secs_f32()
    );

    Ok(())
}
