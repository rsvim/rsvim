//! Range tree.

use crate::prelude::*;
use std::ops::Range;

#[derive(Clone, Default, Debug)]
/// RangeTree is a specialized BTreeMap, which uses [`Range`] as its key. A
/// range can be split into two or three new ranges if any insertion overlaps,
/// new value will override old value on the same range. A position only
/// belongs to one range.
pub struct RangeTree<K, V>
where
  K: geo::CoordNum + min_max_traits::Max + Ord,
  V: Clone,
{
  map: BTreeMap<(K, K), V>,
}

impl<K, V> RangeTree<K, V>
where
  K: geo::CoordNum + min_max_traits::Max + Ord,
  V: Clone,
{
  pub fn new() -> Self {
    Self {
      map: BTreeMap::new(),
    }
  }

  /// Insert new range and value.
  ///
  /// If this range overlaps with existing range, the value of overlapped part
  /// will be override.
  ///
  /// # Time Complexity
  ///
  /// `O(k log n)`, `k` is the count of overlap, `n` is total count of ranges.
  pub fn insert(&mut self, range: Range<K>, value: V) {
    debug_assert!(range.start < range.end);

    // collect all ranges, include overlap and neighbor.
    let mut to_remove = Vec::new();
    let mut to_insert = Vec::new();

    // only query ranges that can overlap.
    // i.e. `start < range.end && end > range.start`.

    let candidate_range =
      self.map.range((range.start, K::MAX)..(range.end, K::MAX));

    for (&(start, end), value) in candidate_range {
      // check if overlap: `range.start < end && start < range.end`
      // since we already limit `start < range.end`, here only need to check
      // `range.start < end`
      if range.start < end {
        to_remove.push((start, end));

        // for overlap range
        // left non-overlap part
        if start < range.start {
          to_insert.push(((start, range.start), value.clone()));
        }
        // right non-overlap part
        if range.end < end {
          to_insert.push(((range.end, end), value.clone()));
        }
      }
    }

    // remove been split range
    for key in to_remove {
      self.map.remove(&key);
    }

    // insert newly split range
    for (key, val) in to_insert {
      self.map.insert(key, val.clone());
    }

    // insert new range
    self.map.insert((range.start, range.end), value);
  }

  /// Query value by positiion.
  ///
  /// # Time Complexity
  ///
  /// - Average: `O(k)`, where `k` is the count of `start <= pos`.
  /// - Worst: `O(n)`, where `n` is the count of total ranges.
  pub fn query(&self, pos: K) -> Option<&V> {
    // Since existing ranges are not overlapped, we only need to check the
    // range in `start <= pos`. i.e. we only need to find out the first range
    // that `start <= pos < end`.
    for (&(start, end), value) in self.map.range(..=(pos, K::MAX)).rev() {
      if start <= pos && pos < end {
        return Some(value);
      }
      // If `start <= pos` bug `pos >= end`, early return
      if start <= pos {
        break;
      }
    }
    None
  }

  pub fn iter(&self) -> impl Iterator<Item = (Range<&K>, &V)> + '_ {
    self
      .map
      .iter()
      .map(|((start, end), value)| (start..end, value))
  }
}
