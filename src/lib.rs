/// A node in the B-tree.
struct Node<K: Ord> {
    keys: Vec<K>,
    children: Vec<Node<K>>,
    is_leaf: bool,
}

impl<K: Ord> Node<K> {
    fn new_leaf() -> Self {
        Node {
            keys: Vec::new(),
            children: Vec::new(),
            is_leaf: true,
        }
    }

    fn new_internal() -> Self {
        Node {
            keys: Vec::new(),
            children: Vec::new(),
            is_leaf: false,
        }
    }
}

/// B-tree with minimum degree `t`. Each non-root node has [t-1, 2t-1] keys.
pub struct BTree<K: Ord> {
    root: Option<Node<K>>,
    t: usize,
    size: usize,
}

impl<K: Ord> BTree<K> {
    /// Create a new B-tree with minimum degree `t`. Panics if `t < 2`.
    pub fn new(t: usize) -> Self {
        assert!(t >= 2, "minimum degree t must be >= 2");
        BTree {
            root: None,
            t,
            size: 0,
        }
    }

    /// Returns true if the key was newly inserted, false if it was already present.
    pub fn insert(&mut self, key: K) -> bool {
        let t = self.t;
        let max_keys = 2 * t - 1;

        if self.root.is_none() {
            let mut leaf = Node::new_leaf();
            leaf.keys.push(key);
            self.root = Some(leaf);
            self.size += 1;
            return true;
        }

        // If root is full, split it
        if self.root.as_ref().unwrap().keys.len() == max_keys {
            let old_root = self.root.take().unwrap();
            let mut new_root = Node::new_internal();
            new_root.children.push(old_root);
            split_child(&mut new_root, 0, t);
            let inserted = insert_non_full(&mut new_root, key, t);
            self.root = Some(new_root);
            if inserted {
                self.size += 1;
            }
            return inserted;
        }

        let inserted = insert_non_full(self.root.as_mut().unwrap(), key, t);
        if inserted {
            self.size += 1;
        }
        inserted
    }

    /// Returns true if the key was found and removed.
    pub fn delete(&mut self, key: &K) -> bool {
        if self.root.is_none() {
            return false;
        }
        let t = self.t;
        let removed = delete_from(self.root.as_mut().unwrap(), key, t);
        if removed {
            self.size -= 1;
            // If root has no keys and has a child, shrink tree
            let root_empty = {
                let root = self.root.as_ref().unwrap();
                root.keys.is_empty() && !root.children.is_empty()
            };
            if root_empty {
                let old_root = self.root.take().unwrap();
                self.root = Some(old_root.children.into_iter().next().unwrap());
            }
        }
        removed
    }

    /// Returns true if the key exists in the tree.
    pub fn search(&self, key: &K) -> bool {
        match &self.root {
            None => false,
            Some(root) => search_node(root, key),
        }
    }

    /// Returns all keys in [low, high], in sorted order.
    pub fn range_query(&self, low: &K, high: &K) -> Vec<&K> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            range_query_rec(root, low, high, &mut result);
        }
        result
    }

    /// Returns the number of keys in the tree.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns true if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns all keys in sorted order.
    pub fn inorder(&self) -> Vec<&K> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            inorder_rec(root, &mut result);
        }
        result
    }

    /// Returns the height of the tree. 0 for empty, 1 for root-only.
    pub fn height(&self) -> usize {
        match &self.root {
            None => 0,
            Some(root) => height_rec(root),
        }
    }
}

impl<K: Ord> Default for BTree<K> {
    fn default() -> Self {
        Self::new(2)
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn split_child<K: Ord>(parent: &mut Node<K>, i: usize, t: usize) {
    let child = &mut parent.children[i];
    // child has 2t-1 keys (indices 0..2t-2)
    // new_child gets keys[t..]  (t-1 keys)
    // median is keys[t-1]
    // child keeps keys[0..t-2] (t-1 keys)
    let is_leaf = child.is_leaf;
    let mut new_child = Node {
        keys: child.keys.drain(t..).collect(),
        children: Vec::new(),
        is_leaf,
    };
    let median = child.keys.pop().unwrap(); // removes key at index t-1

    if !is_leaf {
        new_child.children = child.children.drain(t..).collect();
    }

    parent.keys.insert(i, median);
    parent.children.insert(i + 1, new_child);
}

fn insert_non_full<K: Ord>(node: &mut Node<K>, key: K, t: usize) -> bool {
    let max_keys = 2 * t - 1;

    if node.is_leaf {
        match node.keys.binary_search(&key) {
            Ok(_) => false, // duplicate
            Err(pos) => {
                node.keys.insert(pos, key);
                true
            }
        }
    } else {
        // Find the child to descend into
        let pos = match node.keys.binary_search(&key) {
            Ok(_) => return false, // duplicate found in internal node
            Err(p) => p,
        };

        // If that child is full, split it first
        let child_idx = if node.children[pos].keys.len() == max_keys {
            split_child(node, pos, t);
            // After split, the median is now at node.keys[pos]
            match key.cmp(&node.keys[pos]) {
                std::cmp::Ordering::Equal => return false,
                std::cmp::Ordering::Greater => pos + 1,
                std::cmp::Ordering::Less => pos,
            }
        } else {
            pos
        };

        insert_non_full(&mut node.children[child_idx], key, t)
    }
}

fn search_node<K: Ord>(node: &Node<K>, key: &K) -> bool {
    match node.keys.binary_search(key) {
        Ok(_) => true,
        Err(i) => {
            if node.is_leaf {
                false
            } else {
                search_node(&node.children[i], key)
            }
        }
    }
}

fn inorder_rec<'a, K: Ord>(node: &'a Node<K>, result: &mut Vec<&'a K>) {
    if node.is_leaf {
        for k in &node.keys {
            result.push(k);
        }
    } else {
        for (i, k) in node.keys.iter().enumerate() {
            inorder_rec(&node.children[i], result);
            result.push(k);
        }
        inorder_rec(node.children.last().unwrap(), result);
    }
}

fn height_rec<K: Ord>(node: &Node<K>) -> usize {
    if node.is_leaf {
        1
    } else {
        1 + height_rec(&node.children[0])
    }
}

fn range_query_rec<'a, K: Ord>(node: &'a Node<K>, low: &K, high: &K, result: &mut Vec<&'a K>) {
    let n = node.keys.len();
    for i in 0..n {
        // Recurse into child[i] if it could contain keys in range
        if !node.is_leaf {
            // Only recurse left if low <= node.keys[i] (there might be keys in range on the left)
            if low <= &node.keys[i] {
                range_query_rec(&node.children[i], low, high, result);
            }
        }
        let k = &node.keys[i];
        if k >= low && k <= high {
            result.push(k);
        }
    }
    // Recurse into last child
    if !node.is_leaf {
        range_query_rec(node.children.last().unwrap(), low, high, result);
    }
}

// ── Delete helpers ────────────────────────────────────────────────────────────

fn delete_from<K: Ord>(node: &mut Node<K>, key: &K, t: usize) -> bool {
    let n = node.keys.len();

    match node.keys.binary_search(key) {
        Ok(i) => {
            // Key found in this node
            if node.is_leaf {
                // Case 1: key in leaf
                node.keys.remove(i);
                true
            } else {
                // Case 2: key in internal node
                if node.children[i].keys.len() >= t {
                    // Case 2a: left child has >= t keys, use predecessor
                    let pred = remove_predecessor(&mut node.children[i], t);
                    node.keys[i] = pred;
                    true
                } else if node.children[i + 1].keys.len() >= t {
                    // Case 2b: right child has >= t keys, use successor
                    let succ = remove_successor(&mut node.children[i + 1], t);
                    node.keys[i] = succ;
                    true
                } else {
                    // Case 2c: both children have t-1 keys, merge
                    merge_children(node, i);
                    // key is now in children[i] (the merged node)
                    delete_from(&mut node.children[i], key, t)
                }
            }
        }
        Err(i) => {
            // Key not in this node
            if node.is_leaf {
                return false;
            }
            // Case 3: key might be in children[i]
            // Ensure children[i] has at least t keys
            let child_idx = ensure_child_has_t_keys(node, i, t, n);
            delete_from(&mut node.children[child_idx], key, t)
        }
    }
}

/// Ensures node.children[i] has at least t keys.
/// Returns the possibly-adjusted index to use for recursion.
fn ensure_child_has_t_keys<K: Ord>(node: &mut Node<K>, i: usize, t: usize, n: usize) -> usize {
    if node.children[i].keys.len() >= t {
        return i;
    }

    // Try to borrow from left sibling
    if i > 0 && node.children[i - 1].keys.len() >= t {
        // Rotate right: borrow from left sibling
        rotate_right(node, i);
        return i;
    }

    // Try to borrow from right sibling
    if i < n && node.children[i + 1].keys.len() >= t {
        // Rotate left: borrow from right sibling
        rotate_left(node, i);
        return i;
    }

    // Merge with a sibling
    if i < n {
        // Merge children[i] and children[i+1] with node.keys[i] as separator
        merge_children(node, i);
        i
    } else {
        // Merge children[i-1] and children[i] with node.keys[i-1] as separator
        merge_children(node, i - 1);
        i - 1
    }
}

/// Rotate right: borrow from children[i-1] to children[i] via node.keys[i-1]
fn rotate_right<K: Ord>(node: &mut Node<K>, i: usize) {
    // Move node.keys[i-1] down to children[i].keys[0]
    let sep = node.keys.remove(i - 1);
    node.children[i].keys.insert(0, sep);

    // Move children[i-1].keys.last() up to node.keys[i-1]
    let left_last_key = node.children[i - 1].keys.pop().unwrap();
    node.keys.insert(i - 1, left_last_key);

    // Move last child of left sibling to first child of right
    if !node.children[i - 1].is_leaf {
        let last_child = node.children[i - 1].children.pop().unwrap();
        node.children[i].children.insert(0, last_child);
    }
}

/// Rotate left: borrow from children[i+1] to children[i] via node.keys[i]
fn rotate_left<K: Ord>(node: &mut Node<K>, i: usize) {
    // Move node.keys[i] down to children[i].keys last
    let sep = node.keys.remove(i);
    node.children[i].keys.push(sep);

    // Move children[i+1].keys[0] up to node.keys[i]
    let right_first_key = node.children[i + 1].keys.remove(0);
    node.keys.insert(i, right_first_key);

    // Move first child of right sibling to last child of left
    if !node.children[i + 1].is_leaf {
        let first_child = node.children[i + 1].children.remove(0);
        node.children[i].children.push(first_child);
    }
}

/// Merge children[i] and children[i+1] with node.keys[i] as separator.
/// Result is in children[i]; children[i+1] and keys[i] are removed.
fn merge_children<K: Ord>(node: &mut Node<K>, i: usize) {
    let sep = node.keys.remove(i);
    let right = node.children.remove(i + 1);

    let left = &mut node.children[i];
    left.keys.push(sep);
    left.keys.extend(right.keys);
    left.children.extend(right.children);
}

/// Remove and return the rightmost (predecessor) key from subtree rooted at `node`.
fn remove_predecessor<K: Ord>(node: &mut Node<K>, t: usize) -> K {
    if node.is_leaf {
        node.keys.pop().unwrap()
    } else {
        let last = node.children.len() - 1;
        let n = node.keys.len();
        // Ensure the rightmost child has at least t keys
        if node.children[last].keys.len() < t {
            let _ = ensure_child_has_t_keys(node, last, t, n);
            // After possible merge, last child index might have changed
            let new_last = node.children.len() - 1;
            remove_predecessor(&mut node.children[new_last], t)
        } else {
            remove_predecessor(&mut node.children[last], t)
        }
    }
}

/// Remove and return the leftmost (successor) key from subtree rooted at `node`.
fn remove_successor<K: Ord>(node: &mut Node<K>, t: usize) -> K {
    if node.is_leaf {
        node.keys.remove(0)
    } else {
        let n = node.keys.len();
        // Ensure the leftmost child has at least t keys
        if node.children[0].keys.len() < t {
            let child_idx = ensure_child_has_t_keys(node, 0, t, n);
            remove_successor(&mut node.children[child_idx], t)
        } else {
            remove_successor(&mut node.children[0], t)
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_new_panics_t1() {
        let _ = BTree::<i32>::new(1);
    }

    #[test]
    fn test_new_empty() {
        let bt = BTree::<i32>::new(2);
        assert!(bt.is_empty());
        assert_eq!(bt.len(), 0);
    }

    #[test]
    fn test_insert_search() {
        let mut bt = BTree::new(2);
        bt.insert(5i32);
        assert!(bt.search(&5));
    }

    #[test]
    fn test_insert_new_true() {
        let mut bt = BTree::new(2);
        assert!(bt.insert(42i32));
    }

    #[test]
    fn test_insert_dup_false() {
        let mut bt = BTree::new(2);
        bt.insert(42i32);
        assert!(!bt.insert(42i32));
    }

    #[test]
    fn test_search_miss_empty() {
        let bt = BTree::<i32>::new(2);
        assert!(!bt.search(&5));
    }

    #[test]
    fn test_search_miss_after_inserts() {
        let mut bt = BTree::new(2);
        for i in [1, 2, 3, 4, 6, 7, 8i32] {
            bt.insert(i);
        }
        assert!(!bt.search(&5));
    }

    #[test]
    fn test_inorder_empty() {
        let bt = BTree::<i32>::new(2);
        assert_eq!(bt.inorder(), Vec::<&i32>::new());
    }

    #[test]
    fn test_inorder_sorted() {
        let mut bt = BTree::new(2);
        for k in [5i32, 3, 7, 1, 4, 6, 8] {
            bt.insert(k);
        }
        let result: Vec<i32> = bt.inorder().into_iter().copied().collect();
        assert_eq!(result, vec![1, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_len_insert() {
        let mut bt = BTree::new(2);
        for i in 0..10i32 {
            bt.insert(i);
        }
        assert_eq!(bt.len(), 10);
    }

    #[test]
    fn test_split_root() {
        // With t=2, root can hold at most 3 keys (2t-1=3). Inserting 4 forces a root split.
        let mut bt = BTree::new(2);
        for i in 1..=10i32 {
            bt.insert(i);
        }
        let result: Vec<i32> = bt.inorder().into_iter().copied().collect();
        assert_eq!(result, (1..=10).collect::<Vec<_>>());
        assert_eq!(bt.len(), 10);
    }

    #[test]
    fn test_delete_empty_false() {
        let mut bt = BTree::<i32>::new(2);
        assert!(!bt.delete(&5));
    }

    #[test]
    fn test_delete_leaf_key() {
        let mut bt = BTree::new(2);
        bt.insert(1i32);
        bt.insert(2);
        bt.insert(3);
        assert!(bt.delete(&2));
        assert!(!bt.search(&2));
        assert_eq!(bt.len(), 2);
    }

    #[test]
    fn test_delete_internal_key() {
        let mut bt = BTree::new(2);
        // Insert enough to create internal nodes
        for i in 1..=10i32 {
            bt.insert(i);
        }
        // Delete a key that is likely internal
        assert!(bt.delete(&4));
        assert!(!bt.search(&4));
        assert_eq!(bt.len(), 9);
    }

    #[test]
    fn test_delete_nonexistent_false() {
        let mut bt = BTree::new(2);
        bt.insert(1i32);
        bt.insert(2);
        assert!(!bt.delete(&99));
    }

    #[test]
    fn test_delete_inorder_sorted() {
        let mut bt = BTree::new(2);
        for i in 1..=15i32 {
            bt.insert(i);
        }
        for i in [3i32, 7, 11] {
            bt.delete(&i);
        }
        let result: Vec<i32> = bt.inorder().into_iter().copied().collect();
        let expected: Vec<i32> = (1..=15).filter(|x| ![3, 7, 11].contains(x)).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_range_query_empty() {
        let bt = BTree::<i32>::new(2);
        assert_eq!(bt.range_query(&1, &10), Vec::<&i32>::new());
    }

    #[test]
    fn test_range_query_all() {
        let mut bt = BTree::new(2);
        for i in 1..=5i32 {
            bt.insert(i);
        }
        let result: Vec<i32> = bt.range_query(&1, &5).into_iter().copied().collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_range_query_partial() {
        let mut bt = BTree::new(2);
        for i in 1..=10i32 {
            bt.insert(i);
        }
        let result: Vec<i32> = bt.range_query(&3, &7).into_iter().copied().collect();
        assert_eq!(result, vec![3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_range_query_miss() {
        let mut bt = BTree::new(2);
        for i in 1..=10i32 {
            bt.insert(i);
        }
        assert_eq!(bt.range_query(&20, &30), Vec::<&i32>::new());
    }

    #[test]
    fn test_height_empty() {
        let bt = BTree::<i32>::new(2);
        assert_eq!(bt.height(), 0);
    }

    #[test]
    fn test_height_root_only() {
        let mut bt = BTree::new(2);
        // Insert t-1 = 1 key (won't split root)
        bt.insert(1i32);
        assert_eq!(bt.height(), 1);
    }

    #[test]
    fn test_height_grows() {
        let mut bt = BTree::new(2);
        // With t=2, inserting 4 keys forces root split (height -> 2)
        // Inserting 8 keys forces another level (height -> 3)
        for i in 1..=8i32 {
            bt.insert(i);
        }
        assert!(bt.height() >= 2);
    }

    #[test]
    fn test_t3_insert_delete() {
        let mut bt = BTree::new(3);
        for i in 1..=20i32 {
            bt.insert(i);
        }
        // Delete 10 elements
        for i in [1i32, 3, 5, 7, 9, 11, 13, 15, 17, 19] {
            assert!(bt.delete(&i));
        }
        let result: Vec<i32> = bt.inorder().into_iter().copied().collect();
        let expected: Vec<i32> = (1..=20)
            .filter(|x| ![1, 3, 5, 7, 9, 11, 13, 15, 17, 19].contains(x))
            .collect();
        assert_eq!(result, expected);
        assert_eq!(bt.len(), 10);
    }

    #[test]
    fn test_mass_insert_inorder() {
        let mut bt = BTree::new(2);
        // Deterministic interleaved order: 200, 1, 199, 2, 198, 3, ...
        let n = 200i32;
        let mut lo = 1i32;
        let mut hi = n;
        let mut keys = Vec::new();
        while lo <= hi {
            if lo == hi {
                keys.push(lo);
                break;
            }
            keys.push(hi);
            keys.push(lo);
            lo += 1;
            hi -= 1;
        }
        for k in &keys {
            bt.insert(*k);
        }
        let result: Vec<i32> = bt.inorder().into_iter().copied().collect();
        let expected: Vec<i32> = (1..=200).collect();
        assert_eq!(result, expected);
        assert_eq!(bt.len(), 200);
    }

    #[test]
    fn test_mass_delete_all() {
        let mut bt = BTree::new(2);
        for i in 1..=50i32 {
            bt.insert(i);
        }
        for i in 1..=50i32 {
            assert!(bt.delete(&i), "failed to delete {}", i);
        }
        assert!(bt.is_empty());
        assert_eq!(bt.len(), 0);
    }
}
