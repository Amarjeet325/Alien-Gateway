@echo off
echo Starting Trusted Setup...

if not exist "..\ptau" mkdir "..\ptau"
if not exist "..\build" mkdir "..\build"

snarkjs powersoftau new bn128 12 ..\ptau\pot12_0000.ptau -v

snarkjs powersoftau contribute ..\ptau\pot12_0000.ptau ..\ptau\pot12_0001.ptau ^
  --name="First contribution" -v

snarkjs powersoftau prepare phase2 ..\ptau\pot12_0001.ptau ..\ptau\pot12_final.ptau -v

snarkjs groth16 setup ..\build\hello.r1cs ..\ptau\pot12_final.ptau ..\build\hello_0000.zkey

snarkjs zkey contribute ..\build\hello_0000.zkey ..\build\hello_final.zkey ^
  --name="Final contribution" -v

snarkjs zkey export verificationkey ..\build\hello_final.zkey ..\build\verification_key.json

echo Trusted setup completed!