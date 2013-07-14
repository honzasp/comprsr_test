RUSTC       = rustc

COMPRSR_DIR = ..
ZLIB_DIR    = zlib
SAMPLES_DIR = samples

RUSTC_FLAGS        = -L $(COMPRSR_DIR)
TESTER_FLAGS       = $(RUSTC_FLAGS) --cfg tester --cfg use_colors
BENCHMARKER_FLAGS  = $(RUSTC_FLAGS) --cfg benchmarker
VERBOSE_ZLIB_FLAGS = $(RUSTC_FLAGS)
ZLIB_GEN           = ruby gen_zlib.rb
SAMPLES            = $(shell find $(SAMPLES_DIR) -type f)

ZLIB_TESTER      = tester_zlib
ZLIB_BENCHMARKER = benchmarker_zlib
ZLIB_TESTER_SRC  = tester_zlib.rs
VERBOSE_ZLIB     = verbose_zlib
VERBOSE_ZLIB_SRC = verbose_zlib.rs


.PHONY: all zlib_test clean libs

all: zlib_test

zlib_test: $(ZLIB_TESTER) zlib.dummy
	./$<

zlib_bench: $(ZLIB_BENCHMARKER) zlib.dummy
	./$<

$(VERBOSE_ZLIB): $(VERBOSE_ZLIB_SRC) libs $(COMPRSR_DIR)/libcomprsr_zlib.dummy
	$(RUSTC) $(VERBOSE_ZLIB_FLAGS) $< -o $@

$(ZLIB_TESTER): $(ZLIB_TESTER_SRC) libs $(COMPRSR_DIR)/libcomprsr_zlib.dummy
	$(RUSTC) $(TESTER_FLAGS) $< -o $@

$(ZLIB_BENCHMARKER): $(ZLIB_TESTER_SRC)
	$(RUSTC) $(BENCHMARKER_FLAGS) $< -o $@

zlib.dummy: $(SAMPLES)
	mkdir $(ZLIB_DIR) || true
	$(ZLIB_GEN)
	touch $@

libs:
	$(MAKE) -C $(COMPRSR_DIR) libcomprsr_zlib.dummy

clean:
	rm -rf *.dummy $(ZLIB_DIR) $(ZLIB_TESTER) $(ZLIB_BENCHMARKER)
