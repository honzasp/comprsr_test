RUSTC = rustc
RUSTC_FLAGS = -L ..
ZLIB_GEN = ruby gen_zlib.rb
ZLIB_TESTER = tester_zlib
ZLIB_BENCHMARKER = benchmarker_zlib
ZLIB_TESTER_SRC = tester_zlib.rs
ZLIB_DIR = zlib
SAMPLES_DIR = samples
SAMPLES = $(shell find $(SAMPLES_DIR) -type f)

.PHONY: all zlib_test clean

all: zlib_test

zlib_test: $(ZLIB_TESTER) zlib.dummy
	./$<

zlib_bench: $(ZLIB_BENCHMARKER) zlib.dummy
	./$<

$(ZLIB_TESTER): $(ZLIB_TESTER_SRC)
	$(RUSTC) $(RUSTC_FLAGS) --cfg tester $< -o $@

$(ZLIB_BENCHMARKER): $(ZLIB_TESTER_SRC)
	$(RUSTC) $(RUSTC_FLAGS) --cfg benchmarker $< -o $@

zlib.dummy: $(SAMPLES)
	mkdir $(ZLIB_DIR) || true
	$(ZLIB_GEN)
	touch $@

clean:
	rm -rf *.dummy $(ZLIB_DIR) $(ZLIB_TESTER) $(ZLIB_BENCHMARKER)
