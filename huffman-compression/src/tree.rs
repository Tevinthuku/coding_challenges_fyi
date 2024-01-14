use std::cmp::Ordering;

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct BinarySearchTree {
    weight: usize,
    root: Tree,
}

impl From<(char, usize)> for BinarySearchTree {
    fn from((ch, count): (char, usize)) -> Self {
        let val = Value::Leaf { ch, count };
        BinarySearchTree::new_with_value(val)
    }
}

impl Extend<Value> for BinarySearchTree {
    fn extend<I: IntoIterator<Item = Value>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |elem| {
            self.insert(elem);
        });
    }
}

impl BinarySearchTree {
    fn new_with_value(value: Value) -> Self {
        BinarySearchTree {
            weight: value.weight(),
            root: Tree(Some(Box::new(Node::new(value)))),
        }
    }
    fn insert(&mut self, value: Value) {
        self.root.insert(value)
    }
}

#[derive(Debug, Clone)]
struct Tree(Option<Box<Node>>);

impl Tree {
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

#[derive(Debug, Clone)]
struct Node {
    value: Value,
    left: Tree,
    right: Tree,
}

impl Node {
    pub fn new(value: Value) -> Self {
        Node {
            value,
            left: Tree(None),
            right: Tree(None),
        }
    }
}

impl BinarySearchTree {
    pub fn new(data: impl IntoIterator<Item = (char, usize)>) -> Option<Self> {
        let trees = Trees::from_iter(data.into_iter().map_into());
        trees.merge()
    }
}

fn cmp_tree_by_weight_desc(a: &BinarySearchTree, b: &BinarySearchTree) -> Ordering {
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

struct Trees(Vec<BinarySearchTree>);

impl FromIterator<BinarySearchTree> for Trees {
    fn from_iter<T: IntoIterator<Item = BinarySearchTree>>(iter: T) -> Self {
        let sorted_trees = iter
            .into_iter()
            .sorted_by(cmp_tree_by_weight_desc)
            .collect_vec();

        Self(sorted_trees)
    }
}

impl Trees {
    fn merge(mut self) -> Option<BinarySearchTree> {
        loop {
            let result = self.pop_lowest()?;
            match result {
                PopResult::TreesToMerge {
                    lowest,
                    second_lowest,
                } => {
                    let new_tree_weight = lowest.weight + second_lowest.weight;
                    let mut new_merged_tree =
                        BinarySearchTree::new_with_value(Value::WeightSum(new_tree_weight));
                    new_merged_tree.extend(second_lowest.root.into_sorted_vec());
                    new_merged_tree.extend(lowest.root.into_sorted_vec());

                    self.insert(new_merged_tree);
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

    fn insert(&mut self, tree: BinarySearchTree) {
        self.0.push(tree);
        self.0.sort_unstable_by(cmp_tree_by_weight_desc);
    }
}

enum PopResult {
    TreesToMerge {
        lowest: BinarySearchTree,
        second_lowest: BinarySearchTree,
    },
    Single(BinarySearchTree),
}

#[cfg(test)]
mod tests {
    use crate::tree::BinarySearchTree;

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

        let tree = BinarySearchTree::new(char_mapping).unwrap();
        println!("{tree:?}");
        assert_eq!(tree.weight, 306);
    }
}
