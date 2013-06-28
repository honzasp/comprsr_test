RUSTC = rustc
RUSTC_FLAGS = -L ..
ZLIB_GEN = ruby gen_zlib.rb
ZLIB_TESTER = tester_zlib
ZLIB_TESTER_SRC = tester_zlib.rs
ZLIB_DIR = zlib
SAMPLES_DIR = samples
SAMPLES = $(shell find $(SAMPLES_DIR) -type f)

.PHONY: all zlib_test clean

all: zlib_test

zlib_test: $(ZLIB_TESTER) zlib.dummy
	./$(ZLIB_TESTER)

$(ZLIB_TESTER): $(ZLIB_TESTER_SRC)
	$(RUSTC) $(RUSTC_FLAGS) $< -o $@

zlib.dummy: $(SAMPLES)
	mkdir $(ZLIB_DIR) || true
	$(ZLIB_GEN)
	touch $@

clean:
	rm -rf *.dummy $(ZLIB_DIR)
