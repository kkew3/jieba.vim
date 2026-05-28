TEMPLATE_FILES := $(wildcard *.jieba_test_case.j2)

OUTPUT_TEST_CASES := $(TEMPLATE_FILES:.jieba_test_case.j2=.jieba_test_case)

.PHONY: all clean

all: $(OUTPUT_TEST_CASES)

%.jieba_test_case: %.jieba_test_case.j2 buffers.yaml
	uv run jinja2 -o $@ $^

clean:
	rm -f $(OUTPUT_TEST_CASES)
