use crate::permutation::PermutationKey;

pub fn get_perm_key(p_sign: u32, p_select: u32, mask: u32) -> PermutationKey {
  let positive_sign_mask = p_sign & p_select;
  let negative_sign_mask = (!p_sign & mask) & p_select;
  PermutationKey(positive_sign_mask, negative_sign_mask)
}

pub fn avg(vec: &[f64]) -> f64 {
  let (sum, count) = vec.iter().fold((0.0, 0), |(sum, count), &x| {
      (sum + x, count + 1)
  });

  if count > 0 {
      sum / count as f64
  } else {
      0.0
  }
}
