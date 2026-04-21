CC = gcc
CFLAGS = -O2

SEED ?= v1.seed.c
SRC_C = $(SEED:.seed.c=.c)
TARGET = $(SEED:.seed.c=)

.PHONY: all build run clean

all: run

build:
	python engine.py $(SEED)
	$(CC) $(CFLAGS) $(SRC_C) -o $(TARGET)

run: build
	./$(TARGET)

clean:
	find . -name '*.c' ! -name '*.seed.c' -delete
	rm -f v1 v2
