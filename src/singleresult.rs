use crate::permutation::{Permutation, PermutationKey};
use std::cmp::Ordering;
use std::fmt::Display;

#[derive(Debug)]
pub struct SingleResult {
    pub field_names: Vec<String>,
    pub permutation_sign: u32,
    pub permutation_select: u32,
    pub mask: u32,
    pub diff: f64,
    error: f64,
}

impl Permutation for SingleResult {
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
        self.error
    }

    fn get_mask(&self) -> u32 {
        self.mask
    }

    fn get_diff(&self) -> f64 {
        self.diff
    }
}

impl Display for SingleResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_display(f)
    }
}

impl Eq for SingleResult {}

impl SingleResult {
    pub fn new(
        fields_descr: Vec<String>,
        psign: u32,
        pselect: u32,
        mask: u32,
        diff: f64,
        err: f64,
    ) -> Self {
        SingleResult {
            field_names: fields_descr,
            permutation_sign: psign,
            permutation_select: pselect,
            mask,
            diff,
            error: err,
        }
    }

    pub fn get_own_key(&self) -> PermutationKey {
        self.get_key()
    }
}

impl PartialEq for SingleResult {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
}

impl PartialOrd for SingleResult {
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

impl Ord for SingleResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.perm_cmp(other)
    }
}

impl Default for SingleResult {
    fn default() -> Self {
        Self {
            field_names: Default::default(),
            permutation_sign: Default::default(),
            permutation_select: Default::default(),
            mask: Default::default(),
            diff: f64::MAX,
            error: f64::MAX,
        }
    }
}
