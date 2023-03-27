use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

pub type HashType = u64;
pub type Key = i32;
pub type Value = String;

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    NotFound,
    InvalidProof,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Insert(Key, Value),
    Delete(Key),
    Lookup(Key),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProofNode {
    Left(HashType, Box<ProofNode>),
    Right(Box<ProofNode>, HashType),
    Leaf(Key, Value),
    Empty,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MerkleAvlTree {
    root: Option<Box<Node>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    key: Key,
    value: Value,
    hash: HashType,
    height: i32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl MerkleAvlTree {
    pub fn new() -> MerkleAvlTree {
        MerkleAvlTree { root: None }
    }

    pub fn insert(&mut self, key: Key, value: Value) {
        self.root = Node::insert(self.root.take(), key, value);
    }

    pub fn delete(&mut self, key: Key) -> Result<(), Error> {
        let (new_root, deleted) = Node::delete(self.root.take(), key)?;
        self.root = new_root;
        if deleted.is_some() {
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn lookup(&self, key: Key) -> Result<&Value, Error> {
        Node::lookup(&self.root, key)
    }

    pub fn root_hash(&self) -> Option<HashType> {
        self.root.as_ref().map(|node| node.hash)
    }

    pub fn generate_proof(&self, key: Key) -> Result<ProofNode, Error> {
        Node::generate_proof(&self.root, key)
    }

    pub fn verify_proof(proof: &ProofNode, root_hash: HashType) -> Result<(&Key, &Value), Error> {
        if proof.hash() == root_hash {
            proof.key_value().ok_or(Error::InvalidProof)
        } else {
            Err(Error::InvalidProof)
        }
    }
}

impl Node {
    fn new(key: Key, value: Value) -> Box<Node> {
        Box::new(Node {
            key,
            value: value.clone(),
            hash: Self::compute_hash(&key, &value),
            height: 1,
            left: None,
            right: None,
        })
    }

    fn compute_hash<K: Hash, V: Hash>(key: &K, value: &V) -> HashType {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn height(node: &Option<Box<Node>>) -> i32 {
        node.as_ref().map_or(0, |n| n.height)
    }

    fn update_height_and_hash(node: &mut Box<Node>) {
        node.height = 1 + std::cmp::max(Self::height(&node.left), Self::height(&node.right));
        let mut hasher = DefaultHasher::new();
        node.key.hash(&mut hasher);
        node.value.hash(&mut hasher);
        if let Some(ref left) = node.left {
            left.hash.hash(&mut hasher);
        }
        if let Some(ref right) = node.right {
            right.hash.hash(&mut hasher);
        }
        node.hash = hasher.finish();
    }

    fn balance_factor(node: &Option<Box<Node>>) -> i32 {
        Self::height(&node.as_ref().unwrap().right) - Self::height(&node.as_ref().unwrap().left)
    }

    fn rotate_left(mut node: Box<Node>) -> Box<Node> {
        let mut new_root = node.right.take().unwrap();
        node.right = new_root.left.take();
        new_root.left = Some(node);

        Self::update_height_and_hash(&mut new_root.left.as_mut().unwrap());
        Self::update_height_and_hash(&mut new_root);

        new_root
    }

    fn rotate_right(mut node: Box<Node>) -> Box<Node> {
        let mut new_root = node.left.take().unwrap();
        node.left = new_root.right.take();
        new_root.right = Some(node);

        Self::update_height_and_hash(&mut new_root.right.as_mut().unwrap());
        Self::update_height_and_hash(&mut new_root);

        new_root
    }

    fn balance(node: Option<Box<Node>>) -> Option<Box<Node>> {
        if let Some(mut n) = node {
            let bf = Self::balance_factor(&Some(n.clone()));
            if bf > 1 {
                if Self::balance_factor(&n.right) < 0 {
                    n.right = Some(Self::rotate_right(n.right.take().unwrap()));
                }
                return Some(Self::rotate_left(n));
            } else if bf < -1 {
                if Self::balance_factor(&n.left) > 0 {
                    n.left = Some(Self::rotate_left(n.left.take().unwrap()));
                }
                return Some(Self::rotate_right(n));
            }
            Some(n)
        } else {
            None
        }
    }

    fn insert(node: Option<Box<Node>>, key: Key, value: Value) -> Option<Box<Node>> {
        let node = if let Some(mut n) = node {
            match key.cmp(&n.key) {
                Ordering::Less => {
                    n.left = Self::insert(n.left.take(), key, value);
                }
                Ordering::Greater => {
                    n.right = Self::insert(n.right.take(), key, value);
                }
                Ordering::Equal => {
                    n.value = value;
                }
            }
            Self::update_height_and_hash(&mut n);
            Self::balance(Some(n))
        } else {
            Some(Self::new(key, value))
        };
        node
    }

    fn delete(
        node: Option<Box<Node>>,
        key: Key,
    ) -> Result<(Option<Box<Node>>, Option<Box<Node>>), Error> {
        if let Some(mut n) = node {
            let deleted: Option<Box<Node>>;
            match key.cmp(&n.key) {
                Ordering::Less => {
                    let (new_left, del) = Self::delete(n.left.take(), key)?;
                    n.left = new_left;
                    deleted = del;
                }
                Ordering::Greater => {
                    let (new_right, del) = Self::delete(n.right.take(), key)?;
                    n.right = new_right;
                    deleted = del;
                }
                Ordering::Equal => {
                    deleted = Some(n.clone());
                    if n.left.is_none() {
                        return Ok((n.right.take(), deleted));
                    } else if n.right.is_none() {
                        return Ok((n.left.take(), deleted));
                    } else {
                        let (new_right, min_right) = Self::delete_min(n.right.take().unwrap());
                        n.key = min_right.key;
                        n.value = min_right.value;
                        n.right = new_right;
                    }
                }
            }
            Self::update_height_and_hash(&mut n);
            Ok((Self::balance(Some(n)), deleted))
        } else {
            Err(Error::NotFound)
        }
    }

    fn delete_min(mut node: Box<Node>) -> (Option<Box<Node>>, Box<Node>) {
        if let Some(left) = node.left.take() {
            let (new_left, min_node) = Self::delete_min(left);
            node.left = new_left;
            Self::update_height_and_hash(&mut node);
            (Self::balance(Some(node)), min_node)
        } else {
            (node.right.take(), node)
        }
    }

    fn lookup<'a>(node: &'a Option<Box<Node>>, key: Key) -> Result<&'a Value, Error> {
        if let Some(n) = node {
            match key.cmp(&n.key) {
                Ordering::Less => Self::lookup(&n.left, key),
                Ordering::Greater => Self::lookup(&n.right, key),
                Ordering::Equal => Ok(&n.value),
            }
        } else {
            Err(Error::NotFound)
        }
    }

    fn generate_proof(node: &Option<Box<Node>>, key: Key) -> Result<ProofNode, Error> {
        if let Some(n) = node {
            match key.cmp(&n.key) {
                Ordering::Less => {
                    let left_proof = Self::generate_proof(&n.left, key)?;
                    Ok(ProofNode::Left(n.hash, Box::new(left_proof)))
                }
                Ordering::Greater => {
                    let right_proof = Self::generate_proof(&n.right, key)?;
                    Ok(ProofNode::Right(Box::new(right_proof), n.hash))
                }
                Ordering::Equal => Ok(ProofNode::Leaf(n.key, n.value.clone())),
            }
        } else {
            Ok(ProofNode::Empty)
        }
    }
}
impl ProofNode {
    fn hash(&self) -> HashType {
        match self {
            ProofNode::Left(node_hash, child_proof) => {
                let mut hasher = DefaultHasher::new();
                node_hash.hash(&mut hasher);
                child_proof.hash().hash(&mut hasher);
                hasher.finish()
            }
            ProofNode::Right(child_proof, node_hash) => {
                let mut hasher = DefaultHasher::new();
                child_proof.hash().hash(&mut hasher);
                node_hash.hash(&mut hasher);
                hasher.finish()
            }
            ProofNode::Leaf(key, value) => {
                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                value.hash(&mut hasher);
                hasher.finish()
            }
            ProofNode::Empty => 0,
        }
    }

    fn key_value(&self) -> Option<(&Key, &Value)> {
        match self {
            ProofNode::Leaf(key, value) => Some((key, value)),
            _ => None,
        }
    }
}
