#[cfg(test)]
mod tests {
    use super::*;
    use cassandra_cpp::*;

    fn connect_to_cassandra() -> Result<Session, Error> {
        // Connect to your Cassandra cluster
        let mut cluster = Cluster::default();
        cluster.set_contact_points("127.0.0.1")?;
        let session = cluster.connect()?;

        // Create the schema for the Merkelized AVL Tree
        session.exec(
            "CREATE KEYSPACE IF NOT EXISTS test_merkle_avl_tree
            WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 };",
        )?;
        session.exec(
            "CREATE TABLE IF NOT EXISTS test_merkle_avl_tree.tree_data
            (key int PRIMARY KEY, value text, root_hash bigint);",
        )?;

        Ok(session)
    }

    #[test]
    fn test_insert_lookup_with_cassandra() {
        let session = connect_to_cassandra().unwrap();
        let mut tree = MerkleAvlTree::new();

        // Insert key-value pairs into the tree
        tree.insert(10, "value10".to_string());
        tree.insert(20, "value20".to_string());

        // Insert key-value pairs and root hash into Cassandra
        let root_hash = tree.root_hash().unwrap();
        session.exec(format!(
            "INSERT INTO test_merkle_avl_tree.tree_data (key, value, root_hash) VALUES (10, 'value10', {});",
            root_hash
        )).unwrap();
        session.exec(format!(
            "INSERT INTO test_merkle_avl_tree.tree_data (key, value, root_hash) VALUES (20, 'value20', {});",
            root_hash
        )).unwrap();

        // Retrieve the key-value pairs and root hash from Cassandra
        let result = session
            .exec("SELECT key, value, root_hash FROM test_merkle_avl_tree.tree_data;")
            .unwrap();
        for row in result.iter() {
            let key: i64 = row.get_col(0).unwrap().get_i64().unwrap();
            let value: String = row.get_col(1).unwrap().get_string().unwrap();
            let cassandra_root_hash: i64 = row.get_col(2).unwrap().get_i64().unwrap();

            // Verify the key-value pairs and root hash against the tree
            assert_eq!(tree.lookup(key).unwrap(), value);
            assert_eq!(cassandra_root_hash, root_hash);
        }
    }

    #[test]
    fn test_delete_with_cassandra() {
        let session = connect_to_cassandra().unwrap();
        let mut tree = MerkleAvlTree::new();

        // Insert key-value pairs into the tree
        tree.insert(10, "value10".to_string());
        tree.insert(20, "value20".to_string());

        // Delete a key-value pair from the tree
        tree.delete(10).unwrap();

        // Delete the key-value pair from Cassandra
        session
            .exec("DELETE FROM test_merkle_avl_tree.tree_data WHERE key = 10;")
            .unwrap();

        // Verify the key is deleted
        let result = session
            .exec("SELECT key, value FROM test_merkle_avl_tree.tree_data WHERE key = 10;")
            .unwrap();
        assert!(result.is_empty());
        assert!(tree.lookup(10).is_err());
    }

    #[test]
    fn test_proof_generation_and_verification_with_cassandra() {
        let session = connect_to_cassandra().unwrap();
        let mut tree = MerkleAvlTree::new();

        // Insert key-value pairs into the tree
        tree.insert(10, "value10".to_string());
        tree.insert(20, "value20".to_string());
        tree.insert(5, "value5".to_string());

        // Store the root hash of the tree
        let root_hash = tree.root_hash().unwrap();

        // Generate proof for a key and verify the proof
        let proof = tree.generate_proof(10).unwrap();
        assert!(MerkleAvlTree::verify_proof(&proof, root_hash));

        // Simulate receiving the proof and root_hash from a trusted source
        // This can be done by storing the proof and root_hash in Cassandra
        let proof_from_trusted_source = proof.clone();
        let root_hash_from_trusted_source = root_hash;

        // Verify proof from the trusted source
        assert!(MerkleAvlTree::verify_proof(
            &proof_from_trusted_source,
            root_hash_from_trusted_source
        ));
    }
}
