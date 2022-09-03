for f in ./*.asm; do dotnet ~/Dev/retroassembler/retroassembler.dll $f; done
mv *bin ../bin