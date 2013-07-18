RUSTC       = rustc

COMPRSR_DIR = ..
COMPR_DIR   = compressed
SAMPLES_DIR = samples

RUSTC_FLAGS        = -L $(COMPRSR_DIR)
TESTER_FLAGS       = $(RUSTC_FLAGS) --cfg use_colors
#VERBOSE_ZLIB_FLAGS = $(RUSTC_FLAGS)
GEN_COMPRESSED     = ruby gen_compressed.rb
SAMPLES            = $(shell find $(SAMPLES_DIR) -type f)

TESTER      = tester
TESTER_SRC  = tester.rs
#VERBOSE_ZLIB     = verbose_zlib
#VERBOSE_ZLIB_SRC = verbose_zlib.rs


.PHONY: all test clean libs 

all: test

test: $(TESTER) compressed.dummy
	./$<

#$(VERBOSE_ZLIB): $(VERBOSE_ZLIB_SRC) libs $(COMPRSR_DIR)/libcomprsr_zlib.dummy
#	$(RUSTC) $(VERBOSE_ZLIB_FLAGS) $< -o $@

$(TESTER): $(TESTER_SRC) libs
	$(RUSTC) $(TESTER_FLAGS) $< -o $@

compressed.dummy: $(SAMPLES)
	rm -rf $(COMPR_DIR)
	$(GEN_COMPRESSED)
	touch $@

libs:
	$(MAKE) -C $(COMPRSR_DIR) libcomprsr_zlib.dummy libcomprsr_gzip.dummy

clean:
	rm -f *.dummy $(TESTER)
	rm -rf $(COMPR_DIR) 
