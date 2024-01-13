use std::cmp::Ordering;

use binary_search_tree::BinarySearchTree;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct Tree {
    pub weight: usize,
    inner: BinarySearchTree<Value>,
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

impl Tree {
    pub fn new(data: impl IntoIterator<Item = (char, usize)>) -> Option<Self> {
        let trees = Trees::from_iter(data.into_iter().map_into());
        trees.merge()
    }

    fn merge(mut self, other: Tree) -> Self {
        self.inner.extend(other.inner.into_sorted_vec());
        self.weight += other.weight;
        self
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
    fn cmp_value(&self) -> usize {
        match self {
            Value::WeightSum(val) => *val,
            Value::Leaf { ch: _, count } => *count,
        }
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cmp_value().cmp(&other.cmp_value())
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
                    let new_tree = lowest.merge(second_lowest);
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

        assert_eq!(tree.weight, 306);
    }
}
