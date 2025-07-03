nasm -I ./asm/includes/ ./asm/%1_regs_286.asm -o ./bin/regs_286.bin -l ./asm/regs_286.lst
nasm -I ./asm/includes/ ./asm/%1.asm -o ./bin/program.bin -l ./asm/%1.lst
