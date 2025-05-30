use std::cmp::Ordering;
use crate::utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PermutationKey(pub u32, pub u32);

pub trait Permutation: std::fmt::Display {
    fn get_permutation_sign(&self) -> u32;
    fn get_permutation_select(&self) -> u32;
    fn get_field_names(&self) -> &[String];
    fn get_mask(&self) -> u32;
    fn get_error(&self) -> f64;
    fn get_diff(&self) -> f64;

    fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut permutation_sign = self.get_permutation_sign();
        let mut permutation_select = self.get_permutation_select();
        // reverse because we shift bits to the right when selecting entries
        // so displaying this way gives the natural order (left to right)
        // of payment fields in the order they appear (top to bottom)
        let rev_sign_perm = format!("{:b}", permutation_sign)
            .chars()
            .rev()
            .collect::<String>();
        let rev_sele_perm = format!("{:b}", permutation_select)
            .chars()
            .rev()
            .collect::<String>();
        let _ = writeln!(
            f,
            "permutation_sign: {}, permutation_select: {}, error: {}",
            rev_sign_perm,
            rev_sele_perm,
            self.get_error()
        );

        let mut pretty_formula = String::new();

        self.get_field_names().iter().for_each(|x| {
            let curr_sele = permutation_select & 1;
            let curr_sign = permutation_sign & 1;
            permutation_sign >>= 1;
            permutation_select >>= 1;

            if curr_sele == 0 {
                return;
            }

            let sign_str = if curr_sign == 1 { "+" } else { "-" };
            pretty_formula.push_str(format!(" {} {}", sign_str, x).as_str());
        });
        writeln!(f, "        pretty formula:{}", pretty_formula)
    }

    fn get_key(&self) -> PermutationKey {
        utils::get_perm_key(self.get_permutation_sign(), self.get_permutation_select(), self.get_mask())
    }

    fn perm_cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.get_diff() < other.get_diff() {
            Ordering::Less
        } else if self.get_diff() > other.get_diff() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    fn perm_partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.perm_cmp(other))
    }

    fn perm_lt(&self, other: &Self) -> bool {
        std::matches!(self.perm_partial_cmp(other), Some(Ordering::Less))
    }

    fn perm_le(&self, other: &Self) -> bool {
        std::matches!(
            self.perm_partial_cmp(other),
            Some(Ordering::Less | Ordering::Equal)
        )
    }

    fn perm_gt(&self, other: &Self) -> bool {
        std::matches!(self.perm_partial_cmp(other), Some(Ordering::Greater))
    }

    fn perm_ge(&self, other: &Self) -> bool {
        std::matches!(
            self.perm_partial_cmp(other),
            Some(Ordering::Greater | Ordering::Equal)
        )
    }
}
