//! Range tree map.

use crate::prelude::*;
use std::ops::Range;

#[derive(Debug)]
pub enum IsOverlappedResult {
  Not,
  Left,
  Right,
  Inside,
  Outside,
}

#[derive(Clone, Default, Debug)]
/// RangeTree is a specialized BTreeMap, which uses [`Range`] as its key. A
/// range can be split into two or three new ranges if any insertion overlaps,
/// new value will override old value on the same range. A position only
/// belongs to one range.
pub struct RangeTreeMap<K, V>
where
  K: geo::CoordNum + min_max_traits::Max + Ord,
  V: Clone,
{
  map: BTreeMap<(K, K), V>,
}

impl<K, V> RangeTreeMap<K, V>
where
  K: geo::CoordNum + min_max_traits::Max + Ord,
  V: Clone,
{
  pub fn new() -> Self {
    Self {
      map: BTreeMap::new(),
    }
  }

  #[inline]
  // Case-1:  [b-----{a-a}------b]
  fn _case1<T>(a: &Range<T>, b: &Range<T>) -> bool
  where
    T: geo::CoordNum + min_max_traits::Max + Ord,
  {
    a.start >= b.start && a.end <= b.end
  }

  #[inline]
  // Case-2:  [a----{b--a]------b}
  fn _case2<T>(a: &Range<T>, b: &Range<T>) -> bool
  where
    T: geo::CoordNum + min_max_traits::Max + Ord,
  {
    b.start >= a.start && b.start < a.end
  }

  #[inline]
  /// Whether two ranges `a` and `b` is overlapped.
  ///
  /// It returns:
  /// - Not: not overlapped
  /// - Inside: overlapped, `a` is inside of `b`: [b-----{a-a}------b]
  /// - Outside: overlapped, `a` is outside of `b`: [a-----{b-b}------a]
  /// - Left: overlapped, `a` has left non-overlapped part:
  ///   [a----{b--a]------b}
  /// - Right: overlapped, `a` has right non-overlapped part:
  ///   [b----{a--b]------a}
  pub fn is_overlapped<T>(a: &Range<T>, b: &Range<T>) -> IsOverlappedResult
  where
    T: geo::CoordNum + min_max_traits::Max + Ord,
  {
    debug_assert!(a.start < a.end);
    debug_assert!(b.start < b.end);

    if Self::_case1(a, b) {
      IsOverlappedResult::Inside
    } else if Self::_case1(b, a) {
      IsOverlappedResult::Outside
    } else if Self::_case2(a, b) {
      IsOverlappedResult::Left
    } else if Self::_case2(b, a) {
      IsOverlappedResult::Right
    } else {
      IsOverlappedResult::Not
    }
  }

  #[allow(clippy::type_complexity)]
  fn _diff(
    &mut self,
    range: &Range<K>,
  ) -> (Vec<((K, K), V)>, Vec<((K, K), V)>) {
    debug_assert!(range.start < range.end);

    // collect all ranges, include overlap and neighbor.
    let mut to_remove: Vec<((K, K), V)> = Vec::new();
    let mut to_insert: Vec<((K, K), V)> = Vec::new();

    // only query ranges that can overlap.
    // i.e. `start < range.end && end > range.start`.

    let candidate_range =
      self.map.range((range.start, K::MAX)..(range.end, K::MAX));

    for (&(start, end), value) in candidate_range {
      match Self::is_overlapped(range, &Range { start, end }) {
        IsOverlappedResult::Inside => {
          to_remove.push(((start, end), value.clone()));
          // for left non-overlap part
          to_insert.push(((start, range.start), value.clone()));
          // for right non-overlap part
          to_insert.push(((range.end, end), value.clone()));
        }
        IsOverlappedResult::Outside => {
          to_remove.push(((start, end), value.clone()));
        }
        IsOverlappedResult::Left => {
          to_remove.push(((start, end), value.clone()));
          // for left non-overlap part
          to_insert.push(((start, range.start), value.clone()));
        }
        IsOverlappedResult::Right => {
          to_remove.push(((start, end), value.clone()));
          // for right non-overlap part
          to_insert.push(((range.end, end), value.clone()));
        }
        IsOverlappedResult::Not => {}
      }
    }

    (to_remove, to_insert)
  }

  /// Insert new range and value.
  ///
  /// If this range overlaps with existing range, the value of overlapped part
  /// will be override.
  pub fn insert(&mut self, range: Range<K>, value: V) {
    let (to_remove, to_insert) = self._diff(&range);

    // remove
    for key in to_remove {
      self.map.remove(&key.0);
    }

    // insert split
    for (key, val) in to_insert {
      self.map.insert(key, val.clone());
    }

    // insert newly inserted
    self.map.insert((range.start, range.end), value);
  }

  /// Remove a range and value.
  ///
  /// If this range overlaps with existing range, the value of overlapped part
  /// will also be removed.
  pub fn remove(&mut self, range: &Range<K>) -> Option<Vec<((K, K), V)>> {
    let (to_remove, to_insert) = self._diff(range);

    // remove
    for key in to_remove.iter() {
      self.map.remove(&key.0);
    }

    // insert split
    for (key, val) in to_insert {
      self.map.insert(key, val.clone());
    }

    if to_remove.is_empty() {
      None
    } else {
      Some(to_remove)
    }
  }

  pub fn clear(&mut self) {
    self.map.clear();
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
