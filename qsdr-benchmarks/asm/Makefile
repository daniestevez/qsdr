exes := $(patsubst %.s, %, $(wildcard *.s))
jsons := $(patsubst %.s, %.json, $(wildcard *.s))

.PHONY: all clean benchmark

all: $(exes)

benchmark: $(jsons)

clean:
	rm -f $(exes) *.o *~

%: %.s
	nasm -felf64 $<
	ld -x -o $@ $@.o

%.json: %.s
	./benchmark_loop_alignment.py --source $< --output $@
