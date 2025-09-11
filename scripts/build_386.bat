nasm -I ./asm/includes/ ./asm/%1_regs_386.asm -o ./bin/regs_386.bin -l ./asm/regs_386.lst
nasm -I ./asm/includes/ ./asm/%1.asm -o ./bin/program.bin -l ./asm/%1.lst
