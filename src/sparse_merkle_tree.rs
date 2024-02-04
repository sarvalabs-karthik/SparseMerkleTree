use self::smt_util::hash_branch;
use crate::common::*;
use crate::kv_trait::AuthenticatedKV;
use std::collections::HashMap;

/*
 *  *******************************************
 *                  TASK 5 (type)
 *  *******************************************
 */

mod smt_util {
    use super::*;
    pub fn hash_key(k: &str) -> Digest {
        hash_one_thing("hash_key", k)
    }

    pub fn hash_kv(k: &str, v: &str) -> Digest {
        hash_two_things("hash_kv_K", "hash_kv_V", k, v)
    }

    pub fn hash_branch(l: Digest, r: Digest) -> Digest {
        hash_two_things("hash_branch_L", "hash_branch_R", l, r)
    }

    // root_from_path takes siblings along the path from leaf to merkle root
    // it calculates the digest of the leaf and check's the branch node is left node
    // or right node based on bitstring and then hashes it appropriately untill root node is calculated.
    pub fn root_from_path(path: &[Digest], k: &str, v: &str) -> Digest {
        let mut running_hash = hash_kv(k, v);

        // println!("running hash {:?}", running_hash);

        let h_k: String =
            smt_util::hash_key(k).string().chars().rev().collect();

        for (i, sib) in path.iter().enumerate() {
            if h_k.chars().nth(i).unwrap() == '0' {
                // if leaf is on left then sibling should be on the right
                // println!("hashing {:?} {:?}",running_hash,sib);

                running_hash = hash_branch(running_hash, *sib)
            } else {
                // println!("hashing {:?} {:?}",sib,running_hash);
                running_hash = hash_branch(*sib, running_hash)
            }
        }

        running_hash
    }
}
#[derive(Debug, Clone)]
struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    hash: Digest,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            left: None,
            right: None,
            hash: zero_digest(),
        }
    }
}

#[derive(Debug, Clone)]
struct SparseMerkleTree {
    root: Node,
    store: HashMap<String, String>,
}

#[derive(Debug, Clone)]
enum SparseMerkleTreeProof {
    NotPresent,
    Present { siblings: Vec<Digest> },
}

impl Node {
    // get_proof takes hashed key as string and traverses untill leaf node is reached based on the direction bit
    // after returning from leaf it pushes the sibling of leaf into siblings vector
    // so siblings are captured from leaf on the path to the root
    fn get_proof(&self, h_k: &String, i: u32, siblings: &mut Vec<Digest>) {
        if self.left.is_none() && self.right.is_none() {
            return;
        }

        // if i==255{
        //     return;
        // }

        // println!("{:?} hello {:?} {:?}",i,self.left.as_ref().unwrap().hash,self.right.as_ref().unwrap().hash);

        // println!("hash of left and right {:?}",hash_branch(self.right.as_ref().unwrap().hash,self.left.as_ref().unwrap().hash));
        // println!("actualHash {:?} ",self.hash);
        // println!("");

        // if the child is on left then sibling will be right one, so store right sibling's hash at ith index.
        if h_k.chars().nth((i) as usize).unwrap() == '0' {
            self.left.as_ref().unwrap().get_proof(h_k, i + 1, siblings);
            siblings.push(self.right.as_ref().unwrap().hash);
        } else {
            self.right.as_ref().unwrap().get_proof(h_k, i + 1, siblings);
            siblings.push(self.left.as_ref().unwrap().hash);
        }
    }

    // insert_leaf traverses till leaf based on direction bit,
    // if either of left or right node doesn't exist on the path then a default node with zero digest is created
    // as they are needed to calculate merkle root.
    // Once we reach pre-leaf node, based on the direction bit
    // We create leaf node using kv digest and default sibling node to preleaf.
    // While returning back to root, hash of nodes on path are re-calculated.
    fn insert_leaf(&mut self, h_k: &String, i: u32, h_kv: &Digest) {
        if i == 255 {
            if h_k.chars().nth((i) as usize).unwrap() == '0' {
                self.left = Some(Box::new(Node {
                    left: None,
                    right: None,
                    hash: *h_kv,
                }));

                self.right.get_or_insert_with(Box::default);
            } else {
                self.right = Some(Box::new(Node {
                    left: None,
                    right: None,
                    hash: *h_kv,
                }));

                self.left.get_or_insert_with(Box::default);
            }

            self.hash = hash_branch(
                self.left.as_ref().unwrap().hash,
                self.right.as_ref().unwrap().hash,
            );

            return;
        }

        self.left.get_or_insert_with(Box::default);
        self.right.get_or_insert_with(Box::default);

        if h_k.chars().nth((i) as usize).unwrap() == '0' {
            self.left.as_mut().unwrap().insert_leaf(h_k, i + 1, h_kv);
        } else {
            self.right.as_mut().unwrap().insert_leaf(h_k, i + 1, h_kv);
        }

        self.hash = smt_util::hash_branch(
            self.left.as_ref().unwrap().hash,
            self.right.as_ref().unwrap().hash,
        )
    }

    // remove_leaf traverses till leaf based on direction bit,
    // then leaf hash is set to zero digest.
    // while coming back we check if left and child right child's has zero digest 
    // then we remove those nodes by deallocating memory.
    // we set the node's hash to zero digest after removing childs.
    // Finally we calculate the hash when on of the childs has leaf
    fn remove_leaf(&mut self, h_k: &String, i: u32) {
        if self.left.is_none() && self.right.is_none() {
            self.hash = zero_digest();

            return;
        }

        if h_k.chars().nth((i) as usize).unwrap() == '0' {
            self.left.as_mut().unwrap().remove_leaf(h_k, i + 1);
        } else {
            self.right.as_mut().unwrap().remove_leaf(h_k, i + 1);
        }

        if self.left.as_ref().unwrap().hash == zero_digest()
            && self.right.as_ref().unwrap().hash == zero_digest()
        {
            self.left.take();
            self.right.take();
            self.hash = zero_digest();

            return;
        }

        self.hash = smt_util::hash_branch(
            self.left.as_ref().unwrap().hash,
            self.right.as_ref().unwrap().hash,
        )
    }
}

impl AuthenticatedKV for SparseMerkleTree {
    type K = String;
    type V = String;
    type LookupProof = SparseMerkleTreeProof;
    type Commitment = Digest;

    /*
     *  *******************************************
     *                  TASK 5 (methods)
     *  *******************************************
     */
    fn new() -> Self {
        SparseMerkleTree {
            root: Node {
                left: None,
                right: None,
                hash: zero_digest(),
            },
            store: HashMap::new(),
        }
    }

    // commit returns root node hash as root is calculated in insert function
    fn commit(&self) -> Self::Commitment {
        self.root.hash
    }

    // check_proof checks if merkle root calculated from merkle proof matches the provided commitment
    fn check_proof(
        key: Self::K,
        res: Option<Self::V>,
        pf: &Self::LookupProof,
        comm: &Self::Commitment,
    ) -> Option<()> {
        match (res, pf) {
            (None, SparseMerkleTreeProof::NotPresent) => {}
            (Some(val), SparseMerkleTreeProof::Present { siblings }) => {
                // println!("sibling count {:?}", siblings.len());

                let merkle_root =
                    smt_util::root_from_path(siblings, &key, &val);
                if merkle_root != *comm {
                    // println!("hello {:?}, {:?}", merkle_root, comm);
                    return None;
                }
            }
            _ => return None,
        }

        Some(())
    }

    // get checks if key is present in store, So non-membership of key is proved in O(1) time
    // if key is present value is fetch from store and merkle proof is caculated.
    fn get(&self, key: Self::K) -> (Option<Self::V>, Self::LookupProof) {
        let h_k: String = smt_util::hash_key(&key).string();
        // let h_kv: String = smt_util::hash_kv(&key).string();

        if let Some(val) = self.store.get(&h_k) {
            let mut sib = Vec::new();
            self.root.get_proof(&h_k, 0, &mut sib);

            return (
                Some(val.clone()),
                SparseMerkleTreeProof::Present { siblings: sib },
            );
        }

        (None, SparseMerkleTreeProof::NotPresent)
    }

    /*
     *  *******************************************
     *                  TASK 6
     *  *******************************************
     *
     * insert doesn't insert if both key and value are already present in store
     * if only key exists but value is different then the value is replaced and merkle root is calculated
     * if key doesn't exist the kv pair is inserted in store and merkle root is calculated
     */
    fn insert(self, key: Self::K, value: Self::V) -> Self {
        let mut store = self.store;
        let mut node = self.root;

        let h_k = smt_util::hash_key(&key).string();
        let h_kv = smt_util::hash_kv(&key, &value);

        // if k,v is duplicate the donot insert it.
        if let Some(val) = store.get(&h_k) {
            if *val == value {
                return SparseMerkleTree { root: node, store };
            }
        }

        // println!("insert {:?}{:?}", h_k, value);

        store.insert(h_k.clone(), value.clone());
        node.insert_leaf(&h_k, 0, &h_kv);

        SparseMerkleTree { root: node, store }
    }

    /*
     *  *******************************************
     *                  TASK 6
     *  *******************************************
     * remove checks if key present in store then it will be removed and merkle root is updated
     * otherwise we don't get into merkle tree
     *
     */
    fn remove(self, key: Self::K) -> Self {
        let mut node = self.root;
        let mut store = self.store;

        let h_k = smt_util::hash_key(&key).string();

        // if key not found in store as we don't need to update merkle root
        if store.remove(&h_k.clone()).is_none() {
            return SparseMerkleTree { root: node, store };
        }

        node.remove_leaf(&h_k, 0);

        SparseMerkleTree { root: node, store }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorted_kv::tests::InsertGetRemoveOp;
    use quickcheck::quickcheck;
    use std::collections::HashMap;

    fn hash_smt_insert_get_remove(ops: Vec<InsertGetRemoveOp>) {
        let mut hmap = HashMap::new();
        let mut smt = SparseMerkleTree::new();

        for op in ops {
            match op {
                InsertGetRemoveOp::Insert(k, v) => {
                    hmap.insert(k.clone(), v.clone());
                    smt = smt.insert(k, v);
                }
                InsertGetRemoveOp::Get(k) => {
                    let (val, proof) = smt.get(k.clone());
                    // println!("{:?},{:?}", val, proof);
                    SparseMerkleTree::check_proof(
                        k.clone(),
                        val.clone(),
                        &proof,
                        &smt.commit(),
                    )
                    .unwrap();
                    assert_eq!(hmap.get(&k), val.as_ref());
                }
                InsertGetRemoveOp::Remove(k) => {
                    hmap.remove(&k);
                    smt = smt.remove(k.clone());
                }
            }
        }
    }

    #[quickcheck]
    fn hash_smt_insert_get_quickcheck(ops: Vec<InsertGetRemoveOp>) {
        hash_smt_insert_get_remove(ops);
    }

    #[test]
    fn hash_smt_insert_get_test_cases() {
        use InsertGetRemoveOp::*;

        let test_cases = [
            (
                "query non existing key on empty tree",
                vec![Get("22".to_string())],
            ),
            (
                "query non existing key on non-empty tree",
                vec![
                    Insert("22".to_string(), "".to_string()),
                    Get("23".to_string()),
                ],
            ),
            (
                "query existing key",
                vec![
                    Insert("22".to_string(), "".to_string()),
                    Get("22".to_string()),
                ],
            ),
            (
                "insert duplicate key with same value",
                vec![
                    Insert("22".to_string(), "".to_string()),
                    Insert("22".to_string(), "".to_string()),
                    Get("22".to_string()),
                ],
            ),
            (
                "insert duplicate key with different value",
                vec![
                    Insert("0".to_string(), "".to_string()),
                    Insert("0".to_string(), "\0".to_string()),
                    Get("0".to_string()),
                ],
            ),
            (
                "insert multiple keys and values",
                vec![
                    Insert("80".to_string(), "".to_string()),
                    Insert("9".to_string(), "".to_string()),
                    Insert("9".to_string(), "".to_string()),
                    Insert("9".to_string(), "".to_string()),
                    Insert("80".to_string(), "\u{0}".to_string()),
                    Insert("0".to_string(), "".to_string()),
                    Get("80".to_string()),
                    Get("9".to_string()),
                    Get("0".to_string()),
                ],
            ),
            ("remove non existing key", vec![Remove("22".to_string())]),
            (
                "remove existing key",
                vec![
                    Insert("80".to_string(), "".to_string()),
                    Insert("92".to_string(), "".to_string()),
                    Insert("94".to_string(), "".to_string()),
                    Insert("5".to_string(), "".to_string()),
                    Insert("6".to_string(), "\u{0}".to_string()),
                    Remove("92".to_string()),
                    Get("92".to_string()),
                    Get("94".to_string()),
                    Remove("5".to_string()),
                    Get("6".to_string()),
                ],
            ),
        ];

        for (_, ops) in test_cases {
            hash_smt_insert_get_remove(ops)
        }
    }
}
