Overview
========

This challenge problem is intended to evaluate several important
skills for a software engineer, especially:

(a) Reading, debugging, and writing Rust code
(b) Reasoning about data structures and their implementations
(c) Extracting development requirements from a mathematical and/or
    prose description
(d) Designing tests and using tests to understand code.

A few logistical notes:

- This problem is designed to take 3-4 hours of focused work.
- Please do not spend more than 4 hours on it unless you decide it's
  extremely interesting. We value putting engineers' time to good use,
  and more than 4 hours for an interview problem would be wasteful.
- This problem has several parts. You do not have to complete all
  parts. In particular, we would rather receive a high-quality
  implementation of _most_ parts than a sloppy implementation of all
  parts.
- Feel free to use whatever documentation or search engines might help
  you in this task, but please ensure that the solution you produce is
  your own work, and please do not share the contents of the problem
  or your particular solution publicly.
- The specific problem tasks are labeled with "Task N:".
- We'd recommend working on the tasks in the order they are presented,
  since they are meant to build on each other.

This problem has been provided as a zipped folder. Please make
modifications in the folder and send a new .zip or .tgz back when you
are done, along with a list of which tasks are fully or partially
included, and any special instructions you might have.

Background
==========

Cryptographic Hash Functions
----------------------------

A *Cryptographic Hash Function* `H` is a special type of function that
takes an arbitrarily long sequence of bytes and returns some fixed-size
"digest" of that sequence.

Cryptographic hash functions have two special properties for our purposes:
H
- Preimage resistance: given a digest `d`, it's infeasible to calculate a
  string `s` such that `H(s) = d`.
- Collision resistance: it's infeasible to find two strings `s1` and `s2`
  such that `H(s1) == H(s2)`.

Proving that _any_ function has these properties is extremely difficult --
and in some cases, would be a major theoretical breakthrough! So in
practical usage, well-known hash functions that have resisted many attempts
to break them are _assumed_ to be Preimage- and Collision-resistant.

For this problem, we will be using the `SHA-256` hash function, which has a
256-bit digest.

Authenticated Data Structures
-----------------------------

An *Authenticated Data Structure* (ADS) is a data structure with 2 extra
features:

(a) You can calculate a "commitment" of the data structure, which is a
    small, unique cryptographic representation of the data structure.
(b) Queries about the data structure can provide a "proof" of their
    results, which can be checked against the data structure's digest.

Generally, properties of an authenticated data structure rely on
standard cryptographic assumptions, e.g. that no one can invert or
find collisions in a particular cryptographic hash function.

Commitments must uniquely determine the data structure's value -- ie,
if `a.commit() == b.commit()`, then `a == b`.

"Equal" data structures may not always have equal commitments.
Depending on what `a == b` means, there may be two data structures `a`
and `b` where `a == b` but `a.commit() != b.commit()`. For example, an
array can be used to represent a set, but there are many potential
arrays that can represent the same set!

Proofs for queries must be complete and sound. Completeness means that
every valid query result has a valid proof. Soundness means that valid
proofs imply that the query results are correct. More formally, if you
receive a result `r` for a query `q`, and a proof `pf`, and
`check_proof(r,pf,ads.commit())` succeeds, then `ads.run_query(q) == r`.

A popular example of an ADS is the Git version control system. Git's core
data structure is an authenticated file directory -- the "commitment" is
the commit hash, and queries about files can be "proven" by providing the
other data required (eg, other filenames, those file's hashes, parent
commit hashes, etc.) to calculate the commit hash.

Authenticated Key-Value Stores
------------------------------

An authenticated Key-Value Store is an ADS of an "associative array" or
"map". The methods of the data structure are described in
`src/kv_trait.rs`:

    fn new() -> Self;
    fn commit(&self) -> Self::Commitment;
    fn check_proof(key: Self::K, res: Option<Self::V>, pf: &Self::LookupProof,
        comm: &Self::Commitment) -> Option<()>;

    fn insert(self, key: Self::K, value: Self::V) -> Self;
    fn get(&self, key: Self::K) -> (Option<Self::V>,Self::LookupProof);
    fn remove(self, key: Self::K) -> Self;

`insert`, `get`, and `remove` behave like the same methods in
`std::collections::HashMap`, except that `insert` and `remove` return
copies of the ADS rather than taking `&mut self`.

ADS #1: Sorted K/V Store
========================

The first few tasks revolve around the authenticated KV store
implementation in `src/sorted_kv.rs`. The locations in the file for
Tasks 2 and 3 are marked with comments.

Task 1: Conceptual check
------------------------

Read the contents of `SortedKV::check_proof`, then please describe
briefly your understanding of why `SortedKV::check_proof` is correct.
Put that description in `check_proof.md`. It will help to look at
`SortedKV::get` and `sortedkv_util::root_from_path`.

In particular, think about how `check_proof` should be:

- Complete (easier): Why can you always generate a proof for
  `(key,res,ads.commit())` that `check_proof` accepts if
  `ads.get(key) == res`?
- Sound (trickier): suppose you could compute some `ads: SortedKV`,
  `key: K`, `res: Option<V>`, and `pf: LookupProof` where
  `ads.get(key).0 != res` but
  `SortedKV::check_proof(key,res,pf,ads.commit()).is_some()`. Why
  would this break the assumption that `SHA-256` is a cryptographic
  hash function?

This doesn't need to be a formal correctness proof -- and that
probably isn't worth your time! Just take a stab at understanding and
explaining it -- doing so will help with the later tasks.

Task 2: `insert`/`get` testing
------------------------------

In `src/sorted_kv.rs`, there is a `tests` module, and a test
`hash_btree_insert_get`. This is an example of a powerful type of test
called a "simulation test", where you generate scenarios where two
different data structures or implementations should behave "the same",
and then check that behavior. `hash_btree_insert_get` checks that
`HashMap` and `BTreeMap` behave identically on (parameterized)
sequences of `get`s and `insert`s.

Two tests, `hash_btree_insert_get_quickcheck` and
`hash_btree_insert_get_test_cases`, call `hash_btree_insert_get` to
test it with various inputs.

Implement `hash_sortedkv_insert_get`, so that it tests `SortedKV`'s
`insert`/`get` behavior against `HashMap`'s behavior. Make sure to
also test the unique behavior of `SortedKV` -- ie, whenever you call
`SortedKV::get`, check its proof!

Currently the `hash_sortedkv_insert_get` tests are `#[ignore]`d, but
if you remove that line the test will run through `cargo test`.

If you run:

    $ cargo test hash_sortedkv_insert_get

The tests (`_quickcheck`, and `_test_cases`) will fail if you haven't
yet implemented it. You'll see something like:

    thread 'sorted_kv::tests::hash_sortedkv_insert_get_quickcheck' panicked at '[quickcheck] TEST FAILED (runtime error). Arguments: ([])
    Error: "not yet implemented"', /home/joe/.cargo/registry/src/github.com-1ecc6299db9ec823/quickcheck-1.0.3/src/tester.rs:165:28

If you have a bug in your test, the `Arguments` section of the output
will be a minimal input that fails. Try adding it to `_test_cases` as
a regression test!

Task 3: Find the Bug
--------------------

This implementation of `SortedKV` has a bug in it. Find that bug! Once
you've found the bug, describe the bug and add a test that illustrates
the bug. Describe, but do not implement, a fix.

The _intended_ bug is a logical error, rather than a crash or other
basic correctness issue. Though, if you find such an issue, that works
too!

If you would like a hint, read `task-3-hint.txt`.

ADS #2: Sparse Merkle Tree
==========================

The Sorted K/V Store is a simple data structure, but it has a major
flaw: updates take linear time, since you have to re-calculate all the
hashes in the list.

A more efficient implementation can be built around a single key
realization: bit-strings can be interpreted as paths down binary
trees! Instead of storing `(K,V)` pairs in sorted order, we can store
them in a particular leaf of a binary tree, with the location being
determined by `H(K)`. If `H` is a 256-bit hash, this binary tree has
height 256, and thus will be _overwhelmingly_ empty.

To make things more concrete, a `Sparse Merkle Tree` built on a
cryptographic hash function `H` with an `N`-bit digest is a binary
tree of height `N` such that:

- Leaf nodes are encoded `(K,V)` pairs, located at the leaf node
  determined by reading the bits of `H(K)` as `0 -> left`,
  `1 -> right`. These bits can be read in any order, but that order
  must be consistent.
    - The digest of a leaf node is a hash `H_leaf(K,V) -> Digest`
      (`H_leaf` is built on `H`)
- Subtrees with no leaf nodes are represented as a special "Empty
  subtree", with a special digest (A common choice is the all-`0`
  digest).
- Branch nodes have left and right subtrees, and the digest of a
  branch node is a hash `H_branch(Digest,Digest) -> Digest` of the
  digests of its child subtrees.

The digest of the whole tree is the digest of the root node (thus,
either the all-`0` digest, or some `H_branch`). Lookup proofs are
built on the concept of "sibling hashes" -- ie, for a node `B =
Branch(l,r)`, the sibling hash of `r` is `l.digest()`, and vice versa.
A lookup proof for `(K,None)` is then a list of sibling hashes
following a prefix of `H(K)` which reaches an empty subtree. A lookup
proof for `(K,Some(V))` is a list of sibling hashes following `H(K)`
which ends in `(K,V)`. If the list of sibling hashes is enough to
calculate the root digest of the overall tree, and thus a proof is
valid if and only if it recalculates the correct root digest.

Task 4: Conceptual check
------------------------

Briefly describe in `sparse_mt.md` your understanding of why this data
structure would work. Some good things to touch on would be:

- How do you know that each key is in a unique leaf? (Hint: think
  about hash function properties).
- Which hash function assumption does representing the empty subtree
  by the all-`0` digest rely upon?
- Suppose there were two sparse merkle trees which differ in exactly
  one key, but their digests are the same. Why does this break
  collision-resistance?

Task 5: Basic Implementation
----------------------------

In `src/sparse_merkle_tree.rs`:

Create a `SparseMerkleTree` type which has `String` keys and `String`
values. Implement `commit()`, `get()` and `check_proof()`. Include
some comments justifying why these implementations are correct.

Task 6 (optional): insert() and remove()
----------------

Think about how you would implement the insert() and remove() functions.
Consider performance, in terms of space. We may, depending on time, use
this as a discussion topic during an interview.