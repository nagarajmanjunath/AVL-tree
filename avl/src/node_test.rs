#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_lookup() {
        let mut tree = MerkleAvlTree::new();
        tree.insert(10, "value10".to_string());
        tree.insert(20, "value20".to_string());

        assert_eq!(tree.lookup(10).unwrap(), "value10");
        assert_eq!(tree.lookup(20).unwrap(), "value20");
    }

    #[test]
    fn test_delete() {
        let mut tree = MerkleAvlTree::new();
        tree.insert(10, "value10".to_string());
        tree.insert(20, "value20".to_string());

        tree.delete(10).unwrap();
        assert!(tree.lookup(10).is_err());
    }

    #[test]
    fn test_proof_generation_and_verification() {
        let mut tree = MerkleAvlTree::new();
        tree.insert(10, "value10".to_string());
        tree.insert(20, "value20".to_string());
        tree.insert(5, "value5".to_string());

        let proof = tree.generate_proof(10).unwrap();
        let root_hash = tree.root_hash().unwrap();

        assert!(MerkleAvlTree::verify_proof(&proof, root_hash));
    }

    #[test]
    fn test_failed_proof_verification() {
        let mut tree = MerkleAvlTree::new();
        tree.insert(10, "value10".to_string());
        tree.insert(20, "value20".to_string());
        tree.insert(5, "value5".to_string());

        let proof = tree.generate_proof(10).unwrap();
        let fake_root_hash = 123456789;

        assert!(!MerkleAvlTree::verify_proof(&proof, fake_root_hash));
    }
}
