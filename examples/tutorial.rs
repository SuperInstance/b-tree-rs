//! # B-Tree Tutorial
//!
//! A progressive walkthrough of the `b_tree` crate — a from-scratch B-tree
//! implementation with configurable minimum degree `t`, insert, delete,
//! search, range queries, and inorder traversal.
//!
//! Run:
//!
//!     cargo run --example tutorial

use b_tree_rs::BTree;

fn main() {
    println!("=== B-Tree Tutorial ===\n");

    lesson_1_create_and_insert();
    lesson_2_search();
    lesson_3_inorder_and_len();
    lesson_4_delete();
    lesson_5_range_queries();
    lesson_6_minimum_degree();
    lesson_7_height_and_stress();

    println!("\n✅ All lessons complete!");
}

// ── Lesson 1: Create and Insert ───────────────────────────────────────────────
//
// A B-tree is parameterized by its minimum degree `t`. Every non-root node
// holds between t-1 and 2t-1 keys. When a node overflows, it splits in half
// and pushes its median key up to the parent.
//
// `insert()` returns `true` for new keys, `false` for duplicates.

fn lesson_1_create_and_insert() {
    println!("--- Lesson 1: Create and Insert ---");

    // Create a B-tree with minimum degree t=2 (the classic 2-3-4 tree)
    let mut bt: BTree<i32> = BTree::new(2);

    assert!(bt.is_empty());
    assert_eq!(bt.len(), 0);

    // Insert keys — root splits when it reaches 2t-1 = 3 keys
    println!("  Inserting 1..=6 with t=2 (max 3 keys per node before split):");
    for v in 1..=6 {
        let was_new = bt.insert(v);
        println!("    insert({}) → {} (size={})", v, was_new, bt.len());
    }

    // Duplicate
    assert!(!bt.insert(3));
    println!("  Duplicate insert(3) → false, size still {}", bt.len());

    // t must be >= 2 — constructing with t=1 would panic
    // BTree::<i32>::new(1);  // ← would panic!

    println!();
}

// ── Lesson 2: Search ─────────────────────────────────────────────────────────
//
// `search()` performs O(log n) lookup by binary-searching keys within each
// node and descending into the appropriate child.

fn lesson_2_search() {
    println!("--- Lesson 2: Search ---");

    let mut bt = BTree::new(3); // t=3 → nodes hold [2, 5] keys
    for v in [10, 20, 30, 40, 50, 60, 70, 80, 90] {
        bt.insert(v);
    }

    for &query in &[10, 45, 50, 99] {
        let found = bt.search(&query);
        println!("  search({}) → {}", query, found);
    }

    // Empty tree
    let empty: BTree<i32> = BTree::new(2);
    assert!(!empty.search(&1));
    println!("  Empty tree search(1) → false");

    println!();
}

// ── Lesson 3: In-Order Traversal and Length ───────────────────────────────────
//
// `inorder()` returns all keys in sorted order as `Vec<&K>`. `len()` returns
// the total number of keys. Together they let you verify correctness.

fn lesson_3_inorder_and_len() {
    println!("--- Lesson 3: In-Order Traversal and Length ---");

    let mut bt = BTree::new(2);

    // Insert in random-ish order
    let keys = [50, 10, 80, 5, 30, 60, 90, 15, 45, 70];
    for &k in &keys {
        bt.insert(k);
    }

    let sorted: Vec<i32> = bt.inorder().into_iter().copied().collect();
    println!("  Inserted: {:?}", keys);
    println!("  In-order: {:?}", sorted);
    println!("  Length:   {}", bt.len());

    let mut expected = keys.to_vec();
    expected.sort();
    assert_eq!(sorted, expected);
    assert_eq!(bt.len(), keys.len());

    // Empty tree
    let empty: BTree<i32> = BTree::new(2);
    assert!(empty.inorder().is_empty());
    println!("  Empty tree: inorder=[], len=0");

    println!();
}

// ── Lesson 4: Delete ─────────────────────────────────────────────────────────
//
// B-tree deletion is the trickiest part — it handles three main cases:
//   1. Key in a leaf node → just remove it
//   2. Key in an internal node with a rich child → replace with predecessor/successor
//   3. Key in an internal node with minimal children → merge, then recurse
//
// If underflow occurs, the tree borrows from siblings or merges nodes.
// `delete()` returns `true` if the key was found and removed.

fn lesson_4_delete() {
    println!("--- Lesson 4: Delete ---");

    let mut bt = BTree::new(2);

    for v in 1..=15_i32 {
        bt.insert(v);
    }
    println!("  Inserted 1..=15: inorder={:?}", bt.inorder().into_iter().copied().collect::<Vec<_>>());

    // Delete leaf key
    assert!(bt.delete(&1));
    println!("  delete(1)  [leaf] → true,  size={}", bt.len());

    // Delete internal key
    assert!(bt.delete(&4));
    println!("  delete(4)  [internal] → true, size={}", bt.len());

    // Delete another
    assert!(bt.delete(&10));
    println!("  delete(10) → true,  size={}", bt.len());

    // Nonexistent
    assert!(!bt.delete(&999));
    println!("  delete(999) [not found] → false, size={}", bt.len());

    // Verify sorted order is maintained
    let remaining: Vec<i32> = bt.inorder().into_iter().copied().collect();
    let expected: Vec<i32> = (1..=15).filter(|x| ![1, 4, 10].contains(x)).collect();
    assert_eq!(remaining, expected);
    println!("  Remaining: {:?}", remaining);

    // Delete all remaining → empty tree
    for v in 1..=15_i32 {
        bt.delete(&v);
    }
    assert!(bt.is_empty());
    println!("  Deleted all → is_empty={}", bt.is_empty());

    println!();
}

// ── Lesson 5: Range Queries ──────────────────────────────────────────────────
//
// `range_query(&low, &high)` returns all keys in the inclusive range [low, high]
// in sorted order — without scanning keys outside the range.

fn lesson_5_range_queries() {
    println!("--- Lesson 5: Range Queries ---");

    let mut bt = BTree::new(3);
    for v in 1..=20_i32 {
        bt.insert(v);
    }

    // Full range
    let all: Vec<i32> = bt.range_query(&1, &20).into_iter().copied().collect();
    println!("  range [1, 20] → {} keys", all.len());
    assert_eq!(all.len(), 20);

    // Partial range
    let mid: Vec<i32> = bt.range_query(&5, &10).into_iter().copied().collect();
    println!("  range [5, 10] → {:?}", mid);
    assert_eq!(mid, vec![5, 6, 7, 8, 9, 10]);

    // Single key
    let one: Vec<i32> = bt.range_query(&15, &15).into_iter().copied().collect();
    println!("  range [15, 15] → {:?}", one);
    assert_eq!(one, vec![15]);

    // Range with no matches
    let none: Vec<i32> = bt.range_query(&50, &100).into_iter().copied().collect();
    println!("  range [50, 100] → {:?}", none);
    assert!(none.is_empty());

    // Empty tree range
    let empty: BTree<i32> = BTree::new(2);
    assert!(empty.range_query(&1, &10).is_empty());
    println!("  Empty tree range → []");

    println!();
}

// ── Lesson 6: Minimum Degree (t) Controls Fanout ─────────────────────────────
//
// The minimum degree `t` controls how wide nodes are:
//   • Each node holds between t-1 and 2t-1 keys
//   • Each internal node has between t and 2t children
//
// Higher t → wider, shallower trees → fewer disk reads (in disk-based contexts).
// The Default impl uses t=2.

fn lesson_6_minimum_degree() {
    println!("--- Lesson 6: Minimum Degree (t) ---");

    let n = 100;

    let mut bt2 = BTree::new(2);  // max 3 keys per node
    let mut bt5 = BTree::new(5);  // max 9 keys per node
    let mut bt10 = BTree::new(10); // max 19 keys per node

    for v in 1..=n {
        bt2.insert(v);
        bt5.insert(v);
        bt10.insert(v);
    }

    println!("  Inserted {} keys:", n);
    println!("    t=2  → height={},  keys/node=[1, 3]", bt2.height());
    println!("    t=5  → height={},  keys/node=[4, 9]", bt5.height());
    println!("    t=10 → height={},  keys/node=[9, 19]", bt10.height());

    // All produce the same sorted output
    let sorted: Vec<i32> = (1..=n).collect();
    assert_eq!(bt2.inorder().into_iter().copied().collect::<Vec<_>>(), sorted);
    assert_eq!(bt5.inorder().into_iter().copied().collect::<Vec<_>>(), sorted);
    assert_eq!(bt10.inorder().into_iter().copied().collect::<Vec<_>>(), sorted);

    // Higher t → shorter tree
    assert!(bt10.height() <= bt5.height());
    assert!(bt5.height() <= bt2.height());
    println!("  Confirmed: higher t → shorter tree ✓");

    // Default trait → t=2
    let default_bt: BTree<i32> = BTree::default();
    assert!(default_bt.is_empty());
    println!("  BTree::default() → t=2, empty");

    println!();
}

// ── Lesson 7: Height and Stress Test ─────────────────────────────────────────
//
// B-trees guarantee O(logₜ n) height. With t=2 and 10,000 keys the height
// stays under 20 — far better than a plain BST's worst case of 10,000.

fn lesson_7_height_and_stress() {
    println!("--- Lesson 7: Height and Stress Test (10,000 keys) ---");

    let n: i32 = 10_000;
    let mut bt: BTree<i32> = BTree::new(3);

    // Insert in worst-case sorted order
    for v in 1..=n {
        bt.insert(v);
    }

    let height = bt.height();
    let log_n = (n as f64).log2().ceil() as usize;
    println!("  Inserted {} keys in sorted order with t=3", n);
    println!("  Height: {} (log₂({}) ≈ {})", height, n, log_n);
    assert!(height <= log_n + 2, "B-tree height too large!");

    // Verify full sorted output
    let sorted: Vec<i32> = bt.inorder().into_iter().copied().collect();
    assert_eq!(sorted.len(), n as usize);
    assert_eq!(sorted[0], 1);
    assert_eq!(sorted[n as usize - 1], n);

    // Delete half the keys
    for v in 1..=(n / 2) {
        assert!(bt.delete(&v));
    }
    println!("  After deleting first half: size={}, height={}", bt.len(), bt.height());
    assert_eq!(bt.len(), (n / 2) as usize);

    // Remaining keys are exactly 5001..=10000
    let remaining: Vec<i32> = bt.inorder().into_iter().copied().collect();
    let expected: Vec<i32> = ((n / 2 + 1)..=n).collect();
    assert_eq!(remaining, expected);

    // Range query on remaining
    let range: Vec<i32> = bt.range_query(&7500, &8000).into_iter().copied().collect();
    let range_expected: Vec<i32> = (7500..=8000).collect();
    assert_eq!(range, range_expected);
    println!("  range [7500, 8000] → {} keys ✓", range.len());

    // Delete everything
    for v in (n / 2 + 1)..=n {
        bt.delete(&v);
    }
    assert!(bt.is_empty());
    println!("  Deleted all → is_empty=true");

    println!();
}
