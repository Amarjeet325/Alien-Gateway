const { buildPoseidon } = require("circomlibjs");
const fs = require("fs");

function stringToFieldArray(str, length = 32) {
  const arr = new Array(length).fill(0);
  for (let i = 0; i < str.length && i < length; i++) {
    arr[i] = str.charCodeAt(i);
  }
  return arr;
}

function hashUsername(poseidon, arr) {
  const h = [];
  for (let i = 0; i < 8; i++) {
    h.push(poseidon(arr.slice(i * 4, i * 4 + 4)));
  }
  const h2_0 = poseidon([h[0], h[1], h[2], h[3]]);
  const h2_1 = poseidon([h[4], h[5], h[6], h[7]]);
  return poseidon([h2_0, h2_1]);
}

function buildMerkleTree(poseidon, leaves) {
  let level = leaves;
  const tree = [level];
  while (level.length > 1) {
    const next = [];
    for (let i = 0; i < level.length; i += 2) {
      next.push(poseidon([level[i], level[i + 1]]));
    }
    tree.push(next);
    level = next;
  }
  return tree;
}

(async () => {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;

  const usernames = ["amar", "alice", "bob", "charlie"];

  const leaves = usernames.map((u) =>
    hashUsername(poseidon, stringToFieldArray(u))
  );

  const tree = buildMerkleTree(poseidon, leaves);
  const root = tree[tree.length - 1][0];

  console.log("Merkle root:", F.toString(root));

  const targetIndex = 0; // amar
  let index = targetIndex;

  const pathElements = [];
  const pathIndices = [];

  for (let level = 0; level < tree.length - 1; level++) {
    const siblingIndex = index ^ 1;
    pathElements.push(F.toString(tree[level][siblingIndex]));
    pathIndices.push(index % 2);
    index = Math.floor(index / 2);
  }

  const input = {
    username: stringToFieldArray(usernames[targetIndex]),
    pathElements,
    pathIndices
  };

  fs.writeFileSync(
    "build/merkle_input.json",
    JSON.stringify(input, null, 2)
  );

  console.log("Generated build/merkle_input.json");
})();