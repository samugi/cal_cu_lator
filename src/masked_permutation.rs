pub struct MaskedPermutation {
  mask: u32,
  start: u32,
}

impl MaskedPermutation {
  pub fn new(mask: u32) -> Self {
    MaskedPermutation { mask, start: 0 }
  }
}

impl From<u32> for MaskedPermutation {
  fn from(mask: u32) -> Self {
      MaskedPermutation { mask, start: 0 }
  }
}

impl Iterator for MaskedPermutation {
  type Item = u32;

  fn next(&mut self) -> Option<u32> {
    let ones_count = self.mask.count_ones();
    let upper = (1 << ones_count) as u32;

    if self.start >= upper {
      return None;
    }

    let result = Some(map_to_mask(self.start, self.mask));
    self.start += 1;
    result
  }
}

fn map_to_mask(mut perm: u32, mut mask: u32) -> u32 {
    let mut ret = 0;
    let mut pos = -1;
    loop {
        pos += 1;
        if mask == 0 {
          return ret;
        }

        if mask & 1 == 0 {
          mask >>= 1;
          continue;
        }

        ret |= (perm & 1) << pos;
        perm >>= 1;
        mask >>= 1;
    }
}
