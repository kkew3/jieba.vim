#!/bin/bash

has() {
    while [ -n "$1" ]; do
        if ! builtin command -v "$1" > /dev/null; then
            return 1
        fi
        shift
    done
}

download_release() {
    curr_commit="$(git rev-parse HEAD)"
    curr_tag="$(git tag --points-at "$curr_commit" 2> /dev/null)"
    if [ -z "$curr_tag" ]; then
        return 1
    fi
    arch="$(uname -m)"
    os="$(uname -s)"
    url=""
    case "$arch-$os" in
        x86_64-Darwin)
            url=https://github.com/kkew3/jieba.vim/releases/download/${curr_tag}/jieba_vim_rs-x86_64-apple-darwin.dylib;
            name=jieba_vim_rs.so;
            ;;
        aarch64-Darwin)
            url=https://github.com/kkew3/jieba.vim/releases/download/${curr_tag}/jieba_vim_rs-aarch64-apple-darwin.dylib;
            name=jieba_vim_rs.so;
            ;;
        x86_64-Linux)
            url=https://github.com/kkew3/jieba.vim/releases/download/${curr_tag}/jieba_vim_rs-x86_64-unknown-linux-gnu.so;
            name=jieba_vim_rs.so;
            ;;
    esac
    if [ -z "$url" ]; then
        return 1
    fi
    curl -fsSLo "pythonx/jieba_vim/$name" "$url"
}

build_from_source() {
    local color_when=
    if [ -n "$VIMRUNTIME" ]; then
        color_when=never
    else
        color_when=auto
    fi
    case "$(uname -s)" in
        Darwin)
            cdylib_name=libjieba_vim_rs.dylib
            dest_name=jieba_vim_rs.so
            ;;
        *)
            # Assume that build.sh is never run on Windows.
            cdylib_name=libjieba_vim_rs.so
            dest_name=jieba_vim_rs.so
            ;;
    esac
    # rm: used to delete $dest_name in case it's a symlink
    cd rust_backend \
        && cargo build -r --color=$color_when \
        && rm -f ../pythonx/jieba_vim/$dest_name \
        && cp target/release/$cdylib_name ../pythonx/jieba_vim/$dest_name
}

if has git uname curl; then
    if download_release; then
        exit 0
    fi
fi
build_from_source
