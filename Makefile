RUSTC       = rustc

COMPRSR_DIR = ..
ZLIB_DIR    = zlib
SAMPLES_DIR = samples

RUSTC_FLAGS = -L $(COMPRSR_DIR)
ZLIB_GEN    = ruby gen_zlib.rb
SAMPLES     = $(shell find $(SAMPLES_DIR) -type f)

ZLIB_TESTER      = tester_zlib
ZLIB_BENCHMARKER = benchmarker_zlib
ZLIB_TESTER_SRC  = tester_zlib.rs


.PHONY: all zlib_test clean libs

all: zlib_test

zlib_test: $(ZLIB_TESTER) zlib.dummy
	./$<

zlib_bench: $(ZLIB_BENCHMARKER) zlib.dummy
	./$<

$(ZLIB_TESTER): $(ZLIB_TESTER_SRC) libs $(COMPRSR_DIR)/libcomprsr_zlib.dummy
	$(RUSTC) $(RUSTC_FLAGS) --cfg tester $< -o $@

$(ZLIB_BENCHMARKER): $(ZLIB_TESTER_SRC)
	$(RUSTC) $(RUSTC_FLAGS) --cfg benchmarker $< -o $@

zlib.dummy: $(SAMPLES)
	mkdir $(ZLIB_DIR) || true
	$(ZLIB_GEN)
	touch $@

libs:
	$(MAKE) -C $(COMPRSR_DIR) libcomprsr_zlib.dummy

clean:
	rm -rf *.dummy $(ZLIB_DIR) $(ZLIB_TESTER) $(ZLIB_BENCHMARKER)
