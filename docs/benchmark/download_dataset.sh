#!/bin/bash

# CMRC 2018 dataset
curl -fsSL 'https://raw.githubusercontent.com/ymcui/cmrc2018/refs/heads/master/squad-style-data/cmrc2018_dev.json' \
    | jq -r '.data | .[] | .paragraphs.[0].context' \
    > data_zh.txt
