#!/usr/bin/env bash

# Environment variables:
#
# If JIEBA_VIM_BUILD_FROM_SOURCE=1, then skip downloading cdylib and build from
# source directly.
#
# If JIEBA_VIM_DOWNLOAD_BASE_URL is non-empty, then download cdylib from that
# base url without falling back to building from source. This is intended to be
# used in tests.
#
# If JIEBA_VIM_INSTALL_NVIM=1, then install lua5.1 binding for nvim; otherwise
# install py3 binding for vim. If this variable is not set, install py3 binding
# for vim.

cd "$(dirname -- "${BASH_SOURCE[0]}")" > /dev/null 2>&1

has() {
    while [ -n "$1" ]; do
        if ! builtin command -v "$1" > /dev/null; then
            return 1
        fi
        shift
    done
}

# Define these variables:
#   - DEST_DIR
#   - DEST_NAME
#   - BINDING
#   - ASSET_NAME
#   - LIB_NAME
prepare_release() {
    local lib_stem=
    local lib_ext=
    if [ "$JIEBA_VIM_INSTALL_NVIM" = "1" ]; then
        BINDING=lua51
        DEST_DIR=lua/jieba_vim
        lib_stem=libjieba_vim_jieba_vim_rs
    else
        BINDING=py3
        DEST_DIR=pythonx/jieba_vim
        lib_stem=libjieba_vim_rs
    fi
    local arch="$(uname -m)"
    local os="$(uname -s)"
    ASSET_NAME=
    case "$arch-$os" in
        x86_64-Darwin)
            ASSET_NAME=jieba_vim_rs-x86_64-apple-darwin-$BINDING.dylib
            DEST_NAME=jieba_vim_rs.so
            lib_ext=dylib
            ;;
        aarch64-Darwin | arm64-Darwin)
            ASSET_NAME=jieba_vim_rs-aarch64-apple-darwin-$BINDING.dylib
            DEST_NAME=jieba_vim_rs.so
            lib_ext=dylib
            ;;
        x86_64-Linux | amd64-Linux)
            ASSET_NAME=jieba_vim_rs-x86_64-unknown-linux-gnu-$BINDING.so
            DEST_NAME=jieba_vim_rs.so
            lib_ext=so
            ;;
        aarch64-Linux | arm64-Linux)
            ASSET_NAME=jieba_vim_rs-aarch64-unknown-linux-gnu-$BINDING.so
            DEST_NAME=jieba_vim_rs.so
            lib_ext=so
            ;;
    esac
    if [ -z "$ASSET_NAME" ]; then
        return 1
    fi
    LIB_NAME=$lib_stem.$lib_ext
}

download_release() {
    local curr_commit="$(git rev-parse HEAD)"
    local curr_tag="$(git tag --points-at "$curr_commit" 2> /dev/null)"
    if [ -z "$curr_tag" ]; then
        return 1
    fi
    local url="https://github.com/kkew3/jieba.vim/releases/download/$curr_tag/$ASSET_NAME"
    curl -fsSL -o "$DEST_DIR/$DEST_NAME" "$url"
}

download_release_url() {
    rm -f "$DEST_DIR/$DEST_NAME"
    local url="$JIEBA_VIM_DOWNLOAD_BASE_URL/$ASSET_NAME"
    curl -fsSL -o "$DEST_DIR/$DEST_NAME" "$url"
}

build_from_source() {
    local color_when=
    if [ -n "$VIMRUNTIME" ]; then
        color_when=never
    else
        color_when=auto
    fi
    # rm: used to delete $DEST_NAME in case it's a symlink
    rm -f $DEST_DIR/$DEST_NAME
    cargo clean --color=$color_when --manifest-path rust_backend/Cargo.toml
    cargo build -r --color=$color_when \
        --manifest-path rust_backend/Cargo.toml \
        --package jieba_vim_rs_binding_$BINDING \
        && cp rust_backend/target/release/$LIB_NAME $DEST_DIR/$DEST_NAME
}

prepare_release
if [ -n "$JIEBA_VIM_DOWNLOAD_BASE_URL" ]; then
    if ! has curl; then
        echo "jieba.vim build: cannot download from base url: 'curl' not found" >&2
        exit 1
    fi
    download_release_url
    exit $?
fi
if [ "$JIEBA_VIM_BUILD_FROM_SOURCE" != "1" ] && has git curl; then
    if download_release; then
        exit 0
    fi
fi
if ! has cargo; then
    echo "jieba.vim build: cannot build from source: 'cargo' not found" >&2
    exit 1
fi
build_from_source
