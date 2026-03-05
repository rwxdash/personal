# Build all lessons: make
# Build one lesson: make 01
# Run one lesson:   make run-01
# Clean:            make clean

CC = gcc
CFLAGS = -Wall -Wextra -g -std=c11

# Find all .c files, derive lesson numbers
SOURCES = $(wildcard *.c)
BINS = $(SOURCES:.c=.out)

all: $(BINS)

%.out: %.c
	$(CC) $(CFLAGS) -o $@ $<

# Special: lesson 11 is multi-file
11-headers.out: 11-headers.c 11-math-utils.h
	$(CC) $(CFLAGS) -o $@ 11-headers.c

run-%: %.out
	./$<

compile_commands.json: $(SOURCES)
	@echo '[' > $@
	@first=1; for src in $(SOURCES); do \
		[ $$first -eq 0 ] && echo ',' >> $@; first=0; \
		echo '  {"directory": "$(CURDIR)", "file": "'$$src'", "command": "$(CC) $(CFLAGS) -c '$$src'"}' >> $@; \
	done
	@echo ']' >> $@
	@echo "Generated $@"

.clangd:
	@printf 'CompileFlags:\n  Add:\n    - -xc\n    - -std=gnu11\n    - -Wall\n    - -Wextra\n' > $@
	@echo "Generated $@"

setup: compile_commands.json .clangd
	@echo "Ready"

clean:
	rm -f $(BINS)

.PHONY: all clean setup
