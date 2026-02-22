#!/bin/bash

set -e

echo "Compiling hello.circom..."

circom ../circuits/hello.circom \
  --r1cs --wasm --sym \
  -o ../build

echo "Done! Files in zk/build/"
