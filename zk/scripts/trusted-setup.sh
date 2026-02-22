#!/bin/bash

set -e

echo "Starting Powers of Tau ceremony..."

snarkjs powersoftau new bn128 12 ../ptau/pot12_0000.ptau -v
snarkjs powersoftau contribute ../ptau/pot12_0000.ptau ../ptau/pot12_0001.ptau --name="First contribution" -v
snarkjs powersoftau prepare phase2 ../ptau/pot12_0001.ptau ../ptau/pot12_final.ptau -v

echo "Trusted setup complete!"
