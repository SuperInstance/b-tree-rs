# b-tree-rs

[![crates.io](https://img.shields.io/crates/v/b-tree-rs.svg)](https://crates.io/crates/b-tree-rs)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

B-tree with configurable minimum degree, supporting split/merge, range queries, and all CLRS deletion cases.

## The Problem

You need an ordered map that stays balanced under arbitrary insertions and deletions, with good cache behavior. Binary trees (AVL, red-black) suffer from poor locality — each node holds one key and two pointers, meaning a lookup touches a new cache line at every level. For disk-backed or memory-bound workloads, you want wider nodes that pack multiple keys together.

## The Insight

A B-tree of minimum degree `t` stores between `t-1` and `2t-1` keys per node (except the root). Each node has between `t` and `2t` children. This means:

1. **Shallow trees.** A B-tree with `t=100` stores millions of keys in height 3-4. Fewer levels = fewer pointer dereferences.
2. **Cache-friendly.** Keys within a node are contiguous in memory. A binary search within a node stays in the same cache line.
3. **Balance maintained by structure.** All leaves are at the same depth. No per-node balance factors or color bits.

The cost: nodes can be partially empty (as few as `t-1` keys), and insertion/deletion require careful splitting and merging to maintain the invariant.

## How It Works

### Insertion

1. Search from root to find the correct leaf.
2. If the leaf has room (`< 2t-1` keys), insert in sorted position.
3. If the leaf is full, **split** it: the median key moves up to the parent, and the right half becomes a new sibling. If the parent is also full, split recursively.
4. If the root splits, a new root is created — the tree grows by one level.

The implementation uses the **proactive split** strategy: when descending for insertion, if a child is full, split it *before* entering it. This guarantees no upward propagation is needed (single-pass insertion).

### Deletion

Deletion is the complex part. The CLRS algorithm handles several cases:

1. **Key in a leaf with > t-1 keys:** Simply remove it.
2. **Key in an internal node:** Replace with predecessor (rightmost of left subtree) or successor (leftmost of right subtree), then delete recursively.
3. **Key in a subtree whose root has only t-1 keys:** First ensure the child has at least `t` keys by:
   - **Borrowing** from an adjacent sibling (rotate through parent), or
   - **Merging** with a sibling (combining with parent separator key)
4. If the root ends up empty after a merge, its single child becomes the new root — the tree shrinks.

### Range Query

In-order traversal with pruning: at each node, recurse into child `i` only if `node.keys[i]` could be in `[low, high]`. This skips entire subtrees that fall outside the range.

## Usage

```rust
use b_tree_rs::BTree;

let mut tree = BTree::new(3);  // minimum degree t=3, each node holds 2..5 keys

// Insert — returns false for duplicates
assert!(tree.insert(10));
assert!(tree.insert(20));
assert!(tree.insert(5));
assert!(tree.insert(15));
assert!(!tree.insert(10));  // duplicate

// Search
assert!(tree.search(&15));
assert!(!tree.search(&99));

// Sorted traversal
let sorted: Vec<i32> = tree.inorder().into_iter().copied().collect();
assert_eq!(sorted, vec![5, 10, 15, 20]);

// Range query
let range: Vec<i32> = tree.range_query(&8, &18).into_iter().copied().collect();
assert_eq!(range, vec![10, 15]);

// Delete
assert!(tree.delete(&10));
assert_eq!(tree.len(), 3);

// Height
println!("height: {} (shallow for large t)", tree.height());

// Works with any Ord type
let mut str_tree = BTree::new(2);
str_tree.insert("delta");
str_tree.insert("alpha");
str_tree.insert("charlie");
assert_eq!(str_tree.inorder(), vec![&"alpha", &"charlie", &"delta"]);
```

## Module Map

All types in the crate root (`src/lib.rs`):

| Type | Description |
|---|---|
| `BTree<K: Ord>` | The B-tree. Construct with `new(t)`, minimum degree ≥ 2 |

Internal (private): `Node<K>`, split/merge/rotate helpers, recursive insertion and deletion.

## Design Decisions

- **Minimum degree as parameter.** The caller chooses `t`. For in-memory use, `t=2` (a 2-3-4 tree) is reasonable. For disk-backed or cache-sensitive workloads, larger `t` values (50-200) pack more keys per node.
- **No key-value pairs.** This is a set, not a map. The tree stores `K: Ord` keys only. Extending to `(K, V)` pairs would require storing values alongside keys in each node.
- **Proactive splitting on insert.** Splitting full nodes during descent means insertion is single-pass — no need to unwind the recursion. This is the CLRS recommended approach.
- **In-order successor for deletion.** When deleting an internal node key, the implementation tries the predecessor (left subtree rightmost) first, then the successor (right subtree leftmost). Falls back to merge if both children are at minimum.
- **`Vec`-based node storage.** Keys and children are stored as `Vec<K>` and `Vec<Node<K>>`. Binary search within a node uses `keys.binary_search()`. For small `t`, linear scan would be faster, but `binary_search` scales better with larger `t`.
- **Default `t=2`.** `BTree::default()` creates a tree with minimum degree 2 (2-3-4 tree), matching the most common B-tree variant.

## Complexity

| Operation | Time | Notes |
|---|---|---|
| `insert` | O(t · logₜ n) | Binary search within each node + descent |
| `delete` | O(t · logₜ n) | May involve borrow or merge at each level |
| `search` | O(t · logₜ n) | Binary search per level |
| `range_query` | O(t · logₜ n + k) | k = results in range |
| `inorder` | O(n) | Full traversal |
| `height` | O(logₜ n) | Disk-oriented trees: ~3-4 levels for millions of keys |

For `t=2`, these simplify to O(log n) — the same asymptotic bounds as a red-black tree, but with better cache behavior from contiguous keys.

## Limitations

- **No persistence.** The tree lives in memory only. For disk-backed storage, you'd need serialization and block-aligned node layout.
- **No concurrent operations.** `&mut self` everywhere. Real databases use latch-coupling or optimistic concurrency for B-tree page access.
- **Memory overhead for small `t`.** With `t=2`, each node holds at most 3 keys but uses a full `Vec` (24 bytes heap allocation on 64-bit). A `SmallVec` or fixed-size array would reduce overhead for small `t`.
- **Deletion doesn't compact.** Merged nodes may be half-empty. No rebalancing beyond what's needed for the minimum-key invariant.

## Status

Published to [crates.io](https://crates.io/crates/b-tree-rs). Implements the full CLRS B-tree specification including all deletion cases. Suitable for educational purposes and in-memory ordered sets where you want control over the branching factor. For production use, `std::collections::BTreeMap` is highly optimized and should be preferred.
