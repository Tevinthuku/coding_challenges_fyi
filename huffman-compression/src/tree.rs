use std::{cmp::Ordering, collections::HashMap, error::Error};

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Tree(Option<Box<Node>>);

impl From<(char, usize)> for Tree {
    fn from((ch, count): (char, usize)) -> Self {
        let val = Value::Leaf { ch, count };
        let node = Node::new(val);
        Self(Some(Box::new(node)))
    }
}

impl Tree {
    fn char(&self) -> char {
        self.0
            .as_ref()
            .and_then(|node| match node.value {
                Value::Leaf { ch, count: _ } => Some(ch),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn new(values: impl IntoIterator<Item = (char, usize)>) -> Option<Self> {
        let trees = Trees::from_iter(values.into_iter().map_into());
        trees.merge()
    }

    fn weight(&self) -> usize {
        self.0
            .as_ref()
            .map(|n| n.value.weight())
            .unwrap_or(usize::MIN)
    }

    fn generate_codes(self) -> Result<HashMap<char, CodeAndFrequency>, Box<dyn Error>> {
        let mut result = HashMap::new();
        self.generate_codes_inner(Default::default(), &mut result)?;
        Ok(result)
    }

    fn generate_codes_inner(
        self,
        current_code: String,
        result: &mut HashMap<char, CodeAndFrequency>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(current) = self.0 {
            let left = current.left;
            left.generate_codes_inner(format!("{current_code}0"), result)?;
            if let Value::Leaf { ch, count } = current.value {
                let code = current_code.parse()?;
                result.insert(
                    ch,
                    CodeAndFrequency {
                        frequency: count,
                        code,
                    },
                );
            };
            let right = current.right;
            right.generate_codes_inner(format!("{current_code}1"), result)
        } else {
            Ok(())
        }
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

#[derive(Debug)]
struct CodeAndFrequency {
    frequency: usize,
    code: usize,
}

fn cmp_tree_desc(a: &Tree, b: &Tree) -> Ordering {
    if b.weight() == a.weight() {
        return b.char().cmp(&a.char());
    }
    b.weight().cmp(&a.weight())
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
        let sorted_trees = iter.into_iter().sorted_by(cmp_tree_desc).collect_vec();
        Self(sorted_trees)
    }
}

impl Trees {
    fn merge(mut self) -> Option<Tree> {
        loop {
            let result = self.pop_lowest()?;
            match result {
                PopResult::TreesToMerge { left, right } => {
                    let new_tree_weight = left.weight() + right.weight();

                    let root_node = Node {
                        value: Value::WeightSum(new_tree_weight),
                        right,
                        left,
                    };
                    let tree = Tree(Some(Box::new(root_node)));

                    self.insert(tree);
                }
                PopResult::Single(tree) => {
                    return Some(tree);
                }
            }
        }
    }

    pub fn pop_lowest(&mut self) -> Option<PopResult> {
        let left = self.0.pop()?;

        let item_result = match self.0.pop() {
            Some(right) => PopResult::TreesToMerge { left, right },
            None => PopResult::Single(left),
        };

        Some(item_result)
    }

    fn insert(&mut self, tree: Tree) {
        self.0.push(tree);
        self.0.sort_unstable_by(cmp_tree_desc);
    }
}

enum PopResult {
    TreesToMerge { left: Tree, right: Tree },
    Single(Tree),
}

#[cfg(test)]
mod tests {

    use crate::tree::{CodeAndFrequency, Tree};

    #[test]
    fn test_code_generation() {
        // char_mapping test data comes from
        // https://opendsa-server.cs.vt.edu/ODSA/Books/CS3/html/Huffman.html
        let char_mapping = [
            ('C', 32),
            ('D', 42),
            ('E', 120),
            ('K', 7),
            ('L', 42),
            ('M', 24),
            ('U', 37),
            ('Z', 2),
        ];

        let tree = Tree::new(char_mapping).unwrap();
        assert_eq!(tree.weight(), 306);
        let mut codes = tree.generate_codes().unwrap();
        let expected_codes = [
            ('C', 1110),
            ('D', 101),
            ('E', 0),
            ('K', 111101),
            ('L', 110),
            ('M', 11111),
            ('U', 100),
            ('Z', 111100),
        ];
        for (ch, expected_code) in expected_codes {
            let CodeAndFrequency { code, frequency: _ } = codes.remove(&ch).unwrap();
            assert_eq!(code, expected_code)
        }
    }
}
