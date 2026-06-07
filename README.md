## b-tree-rs

A B-tree implementation in Rust parameterized by minimum degree `t`, supporting insertion with top-down node splitting, deletion via merge and borrow operations, search, and range queries. Each non-root node maintains between `t-1` and `2t-1` keys, ensuring O(log n) time for all operations.
