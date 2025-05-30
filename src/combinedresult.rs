use crate::permutation::Permutation;
use std::cmp::Ordering;
use std::default;
use std::fmt::Display;
use utils::avg;

use crate::utils;

#[derive(Debug, Default)]
pub struct CombinedResult {
    field_names: Vec<String>,
    permutation_sign: u32,
    permutation_select: u32,
    diffs: Vec<f64>,
}

impl Permutation for CombinedResult {
    fn get_permutation_sign(&self) -> u32 {
        self.permutation_sign
    }

    fn get_permutation_select(&self) -> u32 {
        self.permutation_select
    }

    fn get_field_names(&self) -> &[String] {
        &self.field_names
    }

    fn get_error(&self) -> f64 {
        avg(&self.diffs)
    }

    fn get_mask(&self) -> u32 {
        Default::default()
    }

    fn get_diff(&self) -> f64 {
        self.get_error()
    }
}

impl Display for CombinedResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_display(f)
    }
}

impl Eq for CombinedResult {}

impl CombinedResult {
    pub fn new(fields_names: Vec<String>, psign: u32, pselect: u32) -> Self {
        CombinedResult {
            field_names: fields_names,
            permutation_sign: psign,
            permutation_select: pselect,
            diffs: Vec::new(),
        }
    }

    pub fn push_diff(&mut self, diff: f64) {
        self.diffs.push(diff)
    }
}

impl PartialEq for CombinedResult {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
}

impl PartialOrd for CombinedResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn lt(&self, other: &Self) -> bool {
        self.perm_lt(other)
    }

    fn le(&self, other: &Self) -> bool {
        self.perm_le(other)
    }

    fn gt(&self, other: &Self) -> bool {
        self.perm_gt(other)
    }

    fn ge(&self, other: &Self) -> bool {
        self.perm_ge(other)
    }
}

impl Ord for CombinedResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.perm_cmp(other)
    }
}
