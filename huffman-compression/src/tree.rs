use std::cmp::Ordering;

use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct Tree {
    pub weight: usize,
    inner: BinarySearchTree,
}

impl From<(char, usize)> for Tree {
    fn from((ch, count): (char, usize)) -> Self {
        let mut tree = BinarySearchTree::new();
        tree.insert(Value::Leaf { ch, count });
        Self {
            weight: count,
            inner: tree,
        }
    }
}

#[derive(Debug, Clone)]
struct BinarySearchTree {
    root: TreeInner,
}

impl Extend<Value> for BinarySearchTree {
    fn extend<I: IntoIterator<Item = Value>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |elem| {
            self.insert(elem);
        });
    }
}

impl BinarySearchTree {
    pub fn new() -> Self {
        BinarySearchTree {
            root: TreeInner(None),
        }
    }
    pub fn insert(&mut self, value: Value) {
        self.root.insert(value)
    }
}

#[derive(Debug, Clone)]
struct Node {
    value: Value,
    left: TreeInner,
    right: TreeInner,
}

impl Node {
    pub fn new(value: Value) -> Self {
        Node {
            value,
            left: TreeInner(None),
            right: TreeInner(None),
        }
    }
}

#[derive(Debug, Clone)]
struct TreeInner(Option<Box<Node>>);

impl TreeInner {
    pub fn insert(&mut self, value: Value) {
        let mut current = self;

        while let Some(ref mut node) = current.0 {
            match node.value.cmp(&value) {
                Ordering::Greater => current = &mut node.left,
                Ordering::Less => current = &mut node.right,
                Ordering::Equal => {
                    current = &mut node.right;
                }
            }
        }

        current.0 = Some(Box::new(Node::new(value)));
    }

    pub fn into_sorted_vec(self) -> Vec<Value> {
        let mut elements = Vec::new();

        if let Some(node) = self.0 {
            elements.extend(node.left.into_sorted_vec());
            elements.push(node.value);
            elements.extend(node.right.into_sorted_vec());
        }

        elements
    }
}

impl Tree {
    pub fn new(data: impl IntoIterator<Item = (char, usize)>) -> Option<Self> {
        let trees = Trees::from_iter(data.into_iter().map_into());
        trees.merge()
    }
}

pub fn cmp_tree_by_weight_desc(a: &Tree, b: &Tree) -> Ordering {
    b.weight.cmp(&a.weight)
}

#[derive(Clone, Debug)]
enum Value {
    WeightSum(usize),
    Leaf { ch: char, count: usize },
}

impl Value {
    fn weight(&self) -> usize {
        match self {
            Value::WeightSum(val) => *val,
            Value::Leaf { ch: _, count } => *count,
        }
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight().cmp(&other.weight())
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for Value {}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

struct Trees(Vec<Tree>);

impl FromIterator<Tree> for Trees {
    fn from_iter<T: IntoIterator<Item = Tree>>(iter: T) -> Self {
        let sorted_trees = iter
            .into_iter()
            .sorted_by(cmp_tree_by_weight_desc)
            .collect_vec();

        Self(sorted_trees)
    }
}

impl Trees {
    fn merge(mut self) -> Option<Tree> {
        loop {
            let result = self.pop_lowest()?;
            match result {
                PopResult::TreesToMerge {
                    lowest,
                    second_lowest,
                } => {
                    let new_tree_weight = lowest.weight + second_lowest.weight;
                    let mut new_merged_tree = BinarySearchTree::new();
                    new_merged_tree.insert(Value::WeightSum(new_tree_weight));
                    new_merged_tree.extend(second_lowest.inner.root.into_sorted_vec());
                    new_merged_tree.extend(lowest.inner.root.into_sorted_vec());
                    let new_tree = Tree {
                        weight: new_tree_weight,
                        inner: new_merged_tree,
                    };
                    self.insert(new_tree);
                }
                PopResult::Single(tree) => {
                    return Some(tree);
                }
            }
        }
    }

    pub fn pop_lowest(&mut self) -> Option<PopResult> {
        let lowest = self.0.pop()?;

        let item_result = match self.0.pop() {
            Some(second_lowest) => PopResult::TreesToMerge {
                lowest,
                second_lowest,
            },
            None => PopResult::Single(lowest),
        };

        Some(item_result)
    }

    fn insert(&mut self, tree: Tree) {
        self.0.push(tree);
        self.0.sort_unstable_by(cmp_tree_by_weight_desc);
    }
}

enum PopResult {
    TreesToMerge { lowest: Tree, second_lowest: Tree },
    Single(Tree),
}

#[cfg(test)]
mod tests {

    use crate::tree::Tree;

    #[test]
    fn test_merging() {
        // char_mapping test data comes from
        // https://opendsa-server.cs.vt.edu/ODSA/Books/CS3/html/Huffman.html
        let char_mapping = [
            ('Z', 2),
            ('K', 7),
            ('M', 24),
            ('C', 32),
            ('U', 37),
            ('D', 42),
            ('L', 42),
            ('E', 120),
        ];

        let tree = Tree::new(char_mapping).unwrap();
        println!("{tree:?}");
        assert_eq!(tree.weight, 306);
    }
}
