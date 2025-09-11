nasm -I ./asm/includes/ ./asm/regs.asm -o ./bin/regs.bin
nasm -I ./asm/includes/ ./asm/program.asm -o ./bin/program.bin -l ./asm/program.lst
