use std::fmt::Display;
use std::fs::File;
use std::io::Read;

mod combinedresult;
mod item;
mod masked_permutation;
mod permutation;
mod progress;
mod singleresult;
mod sorted_vec;
mod utils;

use combinedresult::CombinedResult;
use item::Item;
use masked_permutation::MaskedPermutation;
use permutation::PermutationKey;
use progress::Progress;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use singleresult::SingleResult;
use sorted_vec::SortedVec;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::thread;

impl Display for SortedVec<SingleResult> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Values:")?;
        for v in &self.data {
            writeln!(f, "\t{}", v)?;
        }
        Ok(())
    }
}

fn get_total_for_perm(permutation_sign: u32, permutation_select: u32, fields: &[Item]) -> f64 {
    let mut total = 0_f64;
    // for each field in the pay slip...
    for (pay_field_n, pay_field) in fields.iter().enumerate() {
        // determine if it should be selected
        let select_field = ((permutation_select >> pay_field_n) & 1) != 0;
        if !select_field {
            continue;
        }

        // determine the sign
        let make_field_negative = ((permutation_sign >> pay_field_n) & 1) == 0;
        for month_amount in &pay_field.values {
            let month_amount_with_sign = if make_field_negative {
                month_amount * -1_f64
            } else {
                *month_amount
            };
            total += month_amount_with_sign;
        }
    }
    total
}

fn find_permutation(
    fields: &[Item],
    goal: f64,
    rank_size: usize,
) -> Result<SortedVec<SingleResult>, String> {
    let num_fields = fields.len();
    if num_fields > 31 {
        return Err("Too many fields (max 31 supported)".to_string());
    }
    let all_fields_mask = (1 << num_fields) - 1;
    println!(
        "Using {:b} mask to compute permutations on {} fields",
        all_fields_mask, num_fields
    );

    let field_names: Vec<String> = fields.iter().map(|i| i.name.clone()).collect();

    let progress = Progress::new(all_fields_mask);

    let rank = (1_u32..=all_fields_mask)
        .into_par_iter()
        .flat_map_iter(|permutation_select| {
            let progress = progress.clone();
            progress.tick();
            MaskedPermutation::from(permutation_select)
                .map(move |permutation_sign| (permutation_select, permutation_sign))
        })
        .map(|(permutation_select, permutation_sign)| {
            let perm_total = get_total_for_perm(permutation_sign, permutation_select, fields);
            let err = perm_total - goal;
            let diff = f64::abs(err);
            SingleResult::new(
                field_names.clone(),
                permutation_sign,
                permutation_select,
                all_fields_mask,
                diff,
                err,
            )
        })
        .fold(
            // This closure is called once per thread to produce a brand-new accumulator:
            || SortedVec::new(rank_size),
            |mut acc, single_result| {
                acc.insert_ordered(single_result);
                acc
            },
        )
        .reduce_with(SortedVec::merged)
        .unwrap();

    Ok(rank)
}

fn run_cu_solver(
    filename: &str,
    goal: f64,
    rank_size: usize,
) -> Result<SortedVec<SingleResult>, String> {
    // Build the CSV reader and iterate over each record.
    let mut file_reader = File::open(filename).expect("not a valid file path");

    let mut file_content = String::new();
    let _ = file_reader.read_to_string(&mut file_content);

    let items: Vec<Item> = file_content
        .lines()
        .map(|line| {
            let mut columns = line.split(",");
            let field_name = columns.next().unwrap();
            let values = columns.map(|c| c.parse::<f64>().unwrap()).collect();
            Item {
                name: field_name.to_string(),
                values,
            }
        })
        .collect();

    let perm_found = find_permutation(&items, goal, rank_size)?;
    Ok(perm_found)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let default_name = "cal-cu-lator".to_string();
    let program = args.first().unwrap_or(&default_name);
    let errmsg =
        "Usage: {} file_path_1.csv goal_1 rank_size_1  [file_path_2.csv goal_2 rank_size_2 ...]";

    let expected_args = 3;
    if (args.len() - 1) % expected_args != 0 {
        panic!("{} {}", errmsg, program);
    }

    let mut thread_handles = vec![];
    let mut index = 0;
    loop {
        index += 1;
        if args.get(index).is_none() {
            break;
        }

        let file = args.get(index).unwrap().clone();
        index += 1;
        let goal: f64 = str::parse(args.get(index).unwrap())
            .unwrap_or_else(|_| panic!("{} {}", errmsg, program));
        index += 1;
        let rank_size: usize = str::parse(args.get(index).unwrap_or(&"10".to_string()))
            .unwrap_or_else(|_| panic!("{} {}", errmsg, program));

        println!(
            "Reading from: {:?}\n\nRunning with goal: {}\nrank_size: {}\n\n",
            file, goal, rank_size
        );
        thread_handles.push(thread::spawn(move || {
            run_cu_solver(file.as_str(), goal, rank_size)
        }));
    }

    let mut file_process_results = Vec::new();
    for handle in thread_handles {
        match handle.join().unwrap() {
            Ok(perm_rank) => file_process_results.push(perm_rank),
            Err(err) => panic!("error running {}: {}", program, err),
        };
    }

    let mut combined_results: HashMap<PermutationKey, CombinedResult> = HashMap::new();

    for res in &file_process_results {
        // join results into combined results
        for candidate in &res.data {
            let combined_result_key: PermutationKey = candidate.get_own_key();

            match combined_results.entry(combined_result_key) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().push_diff(candidate.diff);
                }
                Entry::Vacant(entry) => {
                    let mut comb_res = CombinedResult::new(
                        candidate.field_names.clone(),
                        candidate.permutation_sign,
                        candidate.permutation_select,
                    );
                    comb_res.push_diff(candidate.diff);
                    entry.insert(comb_res);
                }
            }
        }
        println!("\n\nhere is a result {}", res)
    }

    // TODO we can return before the aggregation above probably
    if file_process_results.len() == 1 {
        // nothing to combine
        return;
    }

    // sort the combined results
    let mut sorted_combined_results: SortedVec<CombinedResult> = SortedVec::new(10);
    for cr in combined_results.into_values() {
        sorted_combined_results.insert_ordered(cr);
    }

    for scr in sorted_combined_results.data {
        println!("combined results: {}", scr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_permutation_empty_input() {
        let empty_vec: Vec<Item> = Vec::new();
        let result = find_permutation(&empty_vec, 1000.0, 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 0);
    }

    #[test]
    fn test_find_permutation_from_input_file() {
        let filename = "test_data.csv";
        let goal = 58200.23;
        let rank_size = 10;
        let p_rank_result = run_cu_solver(filename, goal, rank_size);
        assert!(p_rank_result.is_ok());
        let rank = p_rank_result.unwrap();

        let descriptions = vec![
            "AAAAA".to_string(),
            "BBBBB".to_string(),
            "CCCCC".to_string(),
            "DDDDD".to_string(),
            "EEEEE".to_string(),
            "FFFFF".to_string(),
            "ADDED".to_string(),
        ];

        let expected_rank = [
            SingleResult::new(
                descriptions.clone(),
                0b111,
                0b100111,
                0b1111111,
                23.399999999979627,
                23.399999999979627,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b1100111,
                0b1111111,
                0b1111111,
                25.62999999998283,
                -25.62999999998283,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b10,
                0b10110,
                0b1111111,
                29.949999999989814,
                29.949999999989814,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b1110110,
                0b1111110,
                0b1111111,
                45.250000000007276,
                45.250000000007276,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b10,
                0b111,
                0b1111111,
                82.5800000000163,
                -82.5800000000163,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b10110,
                0b110110,
                0b1111111,
                89.13000000002648,
                -89.13000000002648,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b10010,
                0b10111,
                0b111111,
                100.82999999997992,
                100.82999999997992,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b1100110,
                0b1101110,
                0b1111111,
                138.15999999998894,
                -138.15999999998894,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b1100111,
                0b1101111,
                0b1111111,
                157.7800000000134,
                157.7800000000134,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b111,
                0b110111,
                0b1111111,
                160.0100000000166,
                -160.0100000000166,
            ),
        ];

        rank.data.iter().enumerate().for_each(|(i, res)| {
            let b = expected_rank.get(i).unwrap();
            assert_eq!(res, b);
        });
    }

    #[test]
    fn test_find_permutation_from_larger_input_file() {
        let filename = "test_data_larger.csv";
        let goal = 3110.76;
        let rank_size = 3;
        let p_rank_result = run_cu_solver(filename, goal, rank_size);
        assert!(p_rank_result.is_ok());
        let rank = p_rank_result.unwrap();

        let descriptions = vec![
            "AAAAA".to_string(),
            "BBBBB".to_string(),
            "CCCCC".to_string(),
            "DDDDD".to_string(),
            "EEEEE".to_string(),
            "FFFFF".to_string(),
            "GGGGG".to_string(),
            "HHHHH".to_string(),
            "IIIII".to_string(),
            "JJJJJ".to_string(),
        ];

        let expected_rank = [
            SingleResult::new(
                descriptions.clone(),
                0b10010110,
                0b1010111110,
                0b1111111111,
                0.0000000000004547473508864641,
                -0.0000000000004547473508864641,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b11000000,
                0b111110001,
                0b111111111,
                0.15999999999939973,
                0.15999999999939973,
            ),
            SingleResult::new(
                descriptions.clone(),
                0b1010000,
                0b1101011100,
                0b1111111111,
                0.3400000000001455,
                0.3400000000001455,
            ),
        ];

        rank.data.iter().enumerate().for_each(|(i, res)| {
            let b = expected_rank.get(i).unwrap();
            assert_eq!(res, b);
        });
    }
}
