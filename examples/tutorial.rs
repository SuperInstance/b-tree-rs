//! Tutorial: B-tree for disk-friendly sorted storage

use b_tree_rs::BTree;

fn main() {
    println!("=== B-Tree Tutorial ===\n");

    // Part 1: Basic operations with configurable degree
    println!("Part 1: Building a B-tree (min degree t=3)");
    let mut tree: BTree<i32> = BTree::new(3); // min degree = 3, max 5 keys per node
    
    let keys = [100, 50, 150, 25, 75, 125, 175, 10, 60, 90, 35, 200];
    for k in &keys {
        tree.insert(*k);
    }
    println!("  Inserted: {:?}", keys);
    println!("  Size: {}, Height: {}", tree.len(), tree.height());
    println!("  Sorted: {:?}", tree.inorder());
    println!();

    // Part 2: Range queries
    println!("Part 2: Range queries (efficient for large datasets)");
    let range = tree.range_query(&50, &150);
    println!("  Keys in [50, 150]: {:?}", range);
    println!();

    // Part 3: Delete
    println!("Part 3: Delete and rebalance");
    tree.delete(&100);
    tree.delete(&50);
    println!("  After deleting 100, 50:");
    println!("  Size: {}, Height: {}", tree.len(), tree.height());
    println!("  Sorted: {:?}", tree.inorder());
    println!();

    // Part 4: String keys — build queue
    println!("Part 4: Build queue (string keys)");
    let mut build_queue: BTree<String> = BTree::new(2);
    for task in &["wave-1", "wave-10", "wave-2", "wave-21", "wave-3"] {
        build_queue.insert(task.to_string());
    }
    println!("  Tasks (sorted): {:?}", build_queue.inorder());
    let w_range = build_queue.range_query(&"wave-2".to_string(), &"wave-3".to_string());
    println!("  Waves 2-3: {:?}", w_range);
    println!("  Search 'wave-10': {}", build_queue.search(&"wave-10".to_string()));
}
