//! Range tree.

use crate::prelude::*;
use std::ops::Range;

/// RangeTree is a specialized BTreeMap, it uses [`Range`] as its key. And the
/// range can be split into two ranges if new insertion overlaps, new value
/// will override old value on the same range.
pub struct RangeTree {
  // 使用 BTreeMap 存储，key 是 (start, end) 元组，value 是 i32
  // 使用元组而不是 Range 以便自定义排序
  map: BTreeMap<(usize, usize), i32>,
}

impl RangeTree {
  /// 创建一个新的 RangeTree
  pub fn new() -> Self {
    Self {
      map: BTreeMap::new(),
    }
  }

  /// 插入一个区间和对应的值
  /// 如果与现有区间重叠，重叠部分会被新值覆盖
  /// 时间复杂度：O(k log n)，其中 k 是重叠区间的数量，n 是总区间数
  pub fn insert(&mut self, range: Range<usize>, value: i32) {
    if range.start >= range.end {
      return; // 无效区间
    }

    // 收集所有需要处理的区间（包括重叠和相邻的）
    let mut to_remove = Vec::new();
    let mut to_insert = Vec::new();

    // 优化：使用范围查询只检查可能重叠的区间
    // 可能重叠的区间：start < range.end 且 end > range.start
    // 由于 BTreeMap 按 (start, end) 排序，我们可以：
    // 1. 从 start <= range.end 的区间开始查找
    // 2. 只检查 start < range.end 的区间（因为 start >= range.end 的区间不可能重叠）

    // 找到所有 start < range.end 的区间（这些区间可能与新区间重叠）
    let candidate_range = self.map.range(..(range.end, usize::MAX));

    for (&(start, end), &old_value) in candidate_range {
      // 检查是否重叠：range.start < end && start < range.end
      // 由于我们已经限制了 start < range.end，只需检查 range.start < end
      if range.start < end {
        to_remove.push((start, end));

        // 分割区间
        // 左侧非重叠部分
        if start < range.start {
          to_insert.push(((start, range.start), old_value));
        }
        // 右侧非重叠部分
        if range.end < end {
          to_insert.push(((range.end, end), old_value));
        }
      }
    }

    // 移除被分割的区间
    for key in to_remove {
      self.map.remove(&key);
    }

    // 插入分割后的区间
    for (key, val) in to_insert {
      self.map.insert(key, val);
    }

    // 插入新区间
    self.map.insert((range.start, range.end), value);
  }

  /// 获取指定位置的值
  /// 由于插入逻辑会分割重叠区间，最终区间不重叠，所以每个位置最多属于一个区间
  /// 时间复杂度：O(k)，其中 k 是 start <= pos 的区间数量
  /// 最坏情况 O(n)，但实际中由于区间不重叠，k 通常很小
  pub fn get(&self, pos: usize) -> Option<i32> {
    // 由于区间不重叠且按 (start, end) 排序，我们只需要检查 start <= pos 的区间
    // 找到第一个满足 start <= pos < end 的区间
    // 由于区间不重叠，如果某个区间满足 start <= pos 但 pos >= end，
    // 那么所有 start < 该区间 start 的区间也不可能包含 pos
    for (&(start, end), &value) in self.map.range(..=(pos, usize::MAX)).rev() {
      if start <= pos && pos < end {
        return Some(value);
      }
      // 如果 start <= pos 但 pos >= end，由于区间不重叠，
      // 更早的区间（start 更小）也不可能包含 pos，可以提前退出
      if start <= pos {
        break;
      }
    }
    None
  }

  /// 获取所有区间和对应的值
  pub fn iter(&self) -> impl Iterator<Item = (Range<usize>, i32)> + '_ {
    self
      .map
      .iter()
      .map(|(&(start, end), &value)| (start..end, value))
  }

  /// 打印所有区间（用于调试）
  pub fn print_all(&self) {
    for (range, value) in self.iter() {
      println!(
        "key(start:{}, end:{}) = value({})",
        range.start, range.end, value
      );
    }
  }
}

impl Default for RangeTree {
  fn default() -> Self {
    Self::new()
  }
}

fn main() {
  let mut tree = RangeTree::new();

  // 测试用例 1
  println!("=== 测试用例 1 ===");
  tree.insert(10..20, 1);
  println!("插入 key(start:10, end:20) = value(1)");
  tree.print_all();
  println!();

  tree.insert(15..25, 2);
  println!("插入 key(start:15, end:25) = value(2)");
  tree.print_all();
  println!();

  // 重置并测试用例 2
  println!("=== 测试用例 2 ===");
  let mut tree2 = RangeTree::new();
  tree2.insert(10..20, 1);
  tree2.insert(15..25, 2);
  tree2.insert(11..13, 3);
  println!("依次插入:");
  println!("  key(start:10, end:20) = value(1)");
  println!("  key(start:15, end:25) = value(2)");
  println!("  key(start:11, end:13) = value(3)");
  tree2.print_all();
}
