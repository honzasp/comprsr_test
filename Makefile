RUSTC = rustc
RUSTC_FLAGS = -L ..
ZLIB_GEN = ruby zlib/gen.rb
ZLIB_TESTER = zlib/test
ZLIB_TESTER_SRC = zlib/test.rs
SAMPLES_DIR = samples

.PHONY: all zlib_test

all: zlib_test

zlib_test: $(ZLIB_TESTER) zlib_samples.dummy
	./$(ZLIB_TESTER)

$(ZLIB_TESTER): $(ZLIB_TESTER_SRC)
	$(RUSTC) $(RUSTC_FLAGS) $< -o $@

zlib_samples.dummy: $(SAMPLES_DIR)
	$(ZLIB_GEN)
	touch $@
