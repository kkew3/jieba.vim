.PHONY: clean all

TARGETS := demo_vim.gif demo_nvim.gif
VHS ?= vhs

all: $(TARGETS)

clean:
	rm -f $(TARGETS)

%.gif: %.tape
	$(VHS) -o $@ $<
