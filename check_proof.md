check_proof verifies if the value returned by get function for the given key is correct.

Let's take an example where we are querying for value of key(k1) from a sorted kv store, then get function returns index of the leaf, merkle proof which holds sibling hashes along the path from root to leaf and merkle root (commitment) which is basically hash of all key values in store.

root_from_path calculates the merkle root by starting from hash of leaf in upward way by hashing with appropriate sibling. while hashing, sibling is identified as right one if the node is on left side and viceversa (i.e LSB of leaf bit should be 0). LSB is extracted by modulus operator and removed by division with 2. In simple words, they are opposite bit
of available bit on the path to leaf.

Root returned by root_from_path can be compared against commitment to check if the value returned by get function is correct or not. if they are equal it is correct otherwise it isn't.

proof also contains previous leaf and next leaf of current leaf in the sorted kv storeThis function makes few checks which makes sure that previous proof and next proof are correct regardless of whether get method return value associated with key queried. (for example merkle proofs are verified against merkle root)

1. if a value is returned and previous proof is present then it's key should be less than equal to current leaf key as current leaf is right most one and also previous element cannot be absent if leaf index is greater than zero.
2. if a value is returned and next proof is present then it's key should be greater than current key. 
3. if a value is returned and next proof is absent then it means that current leaf is the last node and all of it's right sibling should be empty subtrees
4. if a value is not returned and previous proof is present then check the sorting correctness.
5. if a value is not returned and next proof is present then check the sorting correctness.
6. if a value is not returned and next proof is absent then previous is present then this leaf should be the right most one so check for empty right siblings.


correctness proofs: 

We can always generate a proof such that check proof accepts because hash functions are oneway functions and also they are deterministic.

let's say there is a leaf L and root is R1.It's difficult to find leaf L' such that merkle root is R1. It can only happen if hash function isn't collision resistant. let's say merkle proof for L is hi,hi+1...hj from top to bottom. 

edge cases: If root's are ending up same even though leaves are different it means that same hash is generated for two different nodes along the path
case-1:
we got same hash when we hash leaf hash with sibling node. i.e H(H(L),hj) == H(H(L'),hj)
case-2:
we got same rootnode when we hashed hj with z or z1 (z is the top level hash) i.e H(z,hi) == H(z',hi)

In order for any of the above case to be satisfied SHA-256 shouldn't be collision resistant. Inorder for collisions to happen 2^128 keys are needed (According to Birthday Attack).