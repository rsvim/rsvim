//! Range tree map.

use crate::prelude::*;
use std::fmt::Debug;
use std::ops::Range;

#[derive(Debug)]
pub enum IsOverlappedResult {
  Not,
  Same,
  Left,
  Right,
  Inside,
  Outside,
}

#[derive(Clone, Default, Debug)]
/// RangeTreeMap is a specialized BTreeMap, which uses [`Range`] as its key. A
/// range can be split into two or three new ranges if any insertion overlaps,
/// new value will override old value on the same range. A position only
/// belongs to one range.
pub struct RangeTreeMap<K, V>
where
  K: geo::CoordNum + min_max_traits::Max + min_max_traits::Min + Ord,
  V: Clone,
{
  map: BTreeMap<(K, K), V>,
}

impl<K, V> RangeTreeMap<K, V>
where
  K: geo::CoordNum + min_max_traits::Max + min_max_traits::Min + Ord,
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
  /// - Inside: overlapped, `a` is inside `b`: `[b-----{a-a}------b]`
  /// - Outside: overlapped, `a` is outside `b`: `[a-----{b-b}------a]`
  /// - Left: overlapped, `a` has left non-overlapped part:
  ///   `[a----{b--a]------b}`
  /// - Right: overlapped, `a` has right non-overlapped part:
  ///   `[b----{a--b]------a}`
  pub fn is_overlapped<T>(a: &Range<T>, b: &Range<T>) -> IsOverlappedResult
  where
    T: geo::CoordNum + min_max_traits::Max + Ord,
  {
    debug_assert!(a.start < a.end);
    debug_assert!(b.start < b.end);

    if a == b {
      IsOverlappedResult::Same
    } else if Self::_case1(a, b) {
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

  #[inline]
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

    let candidate_range = self.map.range(..(range.end, K::MAX));

    for (&(start, end), value) in candidate_range {
      match Self::is_overlapped(range, &Range { start, end }) {
        IsOverlappedResult::Inside => {
          to_remove.push(((start, end), value.clone()));
          to_insert.push(((start, range.start), value.clone()));
          to_insert.push(((range.end, end), value.clone()));
        }
        IsOverlappedResult::Outside | IsOverlappedResult::Same => {
          to_remove.push(((start, end), value.clone()));
        }
        IsOverlappedResult::Left => {
          to_remove.push(((start, end), value.clone()));
          to_insert.push(((range.end, end), value.clone()));
        }
        IsOverlappedResult::Right => {
          to_remove.push(((start, end), value.clone()));
          to_insert.push(((start, range.start), value.clone()));
        }
        IsOverlappedResult::Not => {}
      }
    }

    (to_remove, to_insert)
  }

  #[inline]
  /// Insert/set range and value.
  ///
  /// If this range overlaps with existing range, the value of overlapped part
  /// will be overridden.
  pub fn insert(&mut self, range: Range<K>, value: V) {
    let (to_remove, to_insert) = self._diff(&range);

    // remove
    for key in to_remove {
      self.map.remove(&key.0);
    }

    // insert split
    for (key, value) in to_insert {
      self.map.insert(key, value);
    }

    // insert newly inserted
    self.map.insert((range.start, range.end), value);
  }

  #[inline]
  /// Remove/unset range and value.
  ///
  /// If this range overlaps with existing range, the value of overlapped part
  /// will also be removed.
  pub fn remove(&mut self, range: Range<K>) -> Option<Vec<((K, K), V)>> {
    let (to_remove, to_insert) = self._diff(&range);

    // remove
    for key in to_remove.iter() {
      self.map.remove(&key.0);
    }

    // insert split
    for (key, value) in to_insert {
      self.map.insert(key, value);
    }

    if to_remove.is_empty() {
      None
    } else {
      Some(to_remove)
    }
  }

  #[inline]
  pub fn clear(&mut self) {
    self.map.clear();
  }

  #[inline]
  /// Query value by position.
  pub fn query(&self, pos: K) -> Option<&V> {
    for (&(start, end), value) in
      self.map.range((K::MIN, pos)..=(pos, K::MAX)).rev()
    {
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

  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (Range<&K>, &V)> + '_ {
    self
      .map
      .iter()
      .map(|((start, end), value)| (start..end, value))
  }
}
