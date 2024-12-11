/// The enum to describe whether a node is at the border of its parent node.
/// There're several kinds of cases:
///
/// 1. Single-side contact: The node is in contact with its parent on only 1 edge, which looks
///    like:
///
///    ```text
///    -----------------
///    |               |
///    |        -------|
///    |        |//////|
///    |        |//////|
///    |        -------|
///    |               |
///    -----------------
///    ```
///
/// 2. Double-side contact: The node is in contact on 2 edges, which looks like:
///
///    ```text
///    -----------------
///    |        |//////|
///    |        |//////|
///    |        -------|
///    |               |
///    |               |
///    |               |
///    -----------------
///    ```
///
/// 3. Triple-side contact: The node is in contact on 3 edges, which looks like:
///
///    ```text
///    -----------------
///    |  |////////////|
///    |  |////////////|
///    |  |////////////|
///    |  |////////////|
///    |  |////////////|
///    |  |////////////|
///    -----------------
///    ```
///
/// 4. Completely overlapping: The node is in contact on 4 edges, i.e. the node is exactly the same
///    with (or even bigger than, while truncated by) its parent, which looks like:
///
///    ```text
///    -----------------
///    |///////////////|
///    |///////////////|
///    |///////////////|
///    |///////////////|
///    |///////////////|
///    |///////////////|
///    -----------------
///    ```
///
pub enum InodeBorder {
  Top,
  Bottom,
  Left,
  Right,
  TopLeft,
  TopRight,
  BottomLeft,
  BottomRight,
  TopBottomLeft,
  TopBottomRight,
  LeftRightTop,
  LeftRightBottom,
  Overlap,
}

