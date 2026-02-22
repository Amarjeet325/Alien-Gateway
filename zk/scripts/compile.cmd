@echo off
echo Compiling circuit...

if not exist "..\build" mkdir "..\build"

circom ..\circuits\hello.circom ^
  --r1cs ^
  --wasm ^
  --sym ^
  -o ..\build

echo Compilation complete!