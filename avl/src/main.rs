pub mod node;
use crate::node::MerkleAvlTree;
fn main() {
    let mut tree = MerkleAvlTree::new();

    // Insert key-value pairs
    tree.insert(10, "value10".to_string());
    tree.insert(20, "value20".to_string());
    tree.insert(5, "value5".to_string());
    tree.insert(15, "value15".to_string());
    tree.insert(25, "value25".to_string());

    // Lookup keys
    println!("Lookup key 10: {:?}", tree.lookup(10));
    println!("Lookup key 20: {:?}", tree.lookup(20));

    // Compute root hash
    println!("Root hash: {:?}", tree.root_hash());

    // Generate proof for a key and verify the proof
    let proof = tree.generate_proof(10).unwrap();
    let root_hash = tree.root_hash().unwrap();
    println!("Proof for key 10: {:?}", proof);
    println!(
        "Verify proof for key 10: {:?}",
        MerkleAvlTree::verify_proof(&proof, root_hash)
    );

    // Delete a key-value pair
    tree.delete(10).unwrap();

    // Verify if the key is deleted
    println!("Lookup deleted key 10: {:?}", tree.lookup(10));
}
