use std::fmt::Display;
use std::fs::File;
use std::{error::Error, process};

#[derive(Debug, serde::Deserialize)]
struct BustaPagaItems {
    Description: String,
    January: f64,
    February: f64,
    March: f64,
    April: f64,
    May: f64,
    June: f64,
    July: f64,
    August: f64,
    September: f64,
    October: f64,
    November: f64,
    December: f64,
}

struct PermResult {
    permutation_sign: i32,
    permutation_select: i32,
    additional: f64,
    difference: f64,
}

impl PermResult {
    fn new(psign: i32, pselect: i32, add: f64, diff: f64) -> Self {
        PermResult {
            permutation_sign: psign,
            permutation_select: pselect,
            additional: add,
            difference: diff,
        }
    }
}

impl Default for PermResult {
    fn default() -> Self {
        Self {
            permutation_sign: Default::default(),
            permutation_select: Default::default(),
            additional: Default::default(),
            difference: f64::MAX,
        }
    }
}

impl Display for PermResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // reverse because we shift bits to the right when selecting entries
        // so displaying this way gives the natural order (left to right)
        // of payment fields in the order they appear (top to bottom)
        let rev_sign_perm = format!("{:015b}", self.permutation_sign)
            .chars()
            .rev()
            .collect::<String>();
        let rev_sele_perm = format!("{:015b}", self.permutation_select)
            .chars()
            .rev()
            .collect::<String>();
        write!(
            f,
            "permutation_sign: {}, permutation_select: {}, additional: {}, difference: {}",
            rev_sign_perm, rev_sele_perm, self.additional, self.difference
        )
    }
}

fn find_permutation(
    busta_paga: &Vec<BustaPagaItems>,
    goal: f64,
    additional: f64,
) -> Result<PermResult, String> {
    let mut pay_fields = Vec::new();

    // add items to this structure which is more convenient
    // it's a vector of tuples (field_name, Vec<f64>)
    // where the second vector element is the montly value of the field
    for item in busta_paga {
        let mut descr_and_months = (item.Description.clone(), Vec::new());
        descr_and_months.1.extend_from_slice(&[
            item.January,
            item.February,
            item.March,
            item.April,
            item.May,
            item.June,
            item.July,
            item.August,
            item.September,
            item.October,
            item.November,
            item.December,
        ]);
        pay_fields.push(descr_and_months);
    }

    let mut best = PermResult::default();

    let num_fields = pay_fields.len();
    if num_fields > 31 {
        return Err("Too many fields (max 31 supported)".to_string());
    }
    let all_fields_mask = (1 << num_fields) - 1;
    println!("Using {:015b} mask", all_fields_mask);
    let mut perm_count = 0;

    // with or without the additional value (tender offer)
    for additional_val in [0 as f64, additional].iter() {
        // for each field we use this to apply or not the minus sign
        for permutation_sign in 1..=all_fields_mask {
            // for each field we use this to consider or not the field
            for permutation_select in 1..=all_fields_mask {
                let mut total = *additional_val;
                perm_count += 1;

                for (pay_field_n, pay_field) in pay_fields.iter().enumerate() {
                    let select_field = ((permutation_select >> pay_field_n) & 1) != 0;
                    let positive_field = ((permutation_sign >> pay_field_n) & 1) != 0;

                    for month_amount in &pay_field.1 {
                        if select_field {
                            let sign = if positive_field { 1.0 } else { -1.0 };
                            total += sign * month_amount;
                        }
                    }
                }

                let diff = f64::abs(total - goal);
                if diff < best.difference {
                    best = PermResult::new(
                        permutation_sign,
                        permutation_select,
                        *additional_val,
                        diff,
                    );
                }
                if diff == 0.0 {
                    println!(
                        "Returned early for exact match found! After {} permutations",
                        perm_count
                    );
                    return Ok(best);
                }
            }
        }
    }
    println!("Returned at the end, after {} permutations", perm_count);
    Ok(best)
}

fn run_cu_solver(goal: f64, additional: f64) -> Result<(), Box<dyn Error>> {
    // Build the CSV reader and iterate over each record.
    let file_reader = File::open("cucalculator.csv")?;
    let mut rdr = csv::Reader::from_reader(file_reader);

    let busta_paga: Vec<BustaPagaItems> = rdr.deserialize().collect::<Result<_, _>>()?;
    let perm_found = find_permutation(&busta_paga, goal, additional)?;
    println!("Found: {}", perm_found);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let goal: f64 = str::parse(args.get(1).unwrap()).unwrap();
    let additional: f64 = str::parse(args.get(2).unwrap()).unwrap();
    println!("Running with goal: {} stocks sold at: {}", goal, additional);
    if let Err(err) = run_cu_solver(goal, additional) {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
