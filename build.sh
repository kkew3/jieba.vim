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
#   - ASSET_NAME (may be empty)
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
    # Reference: https://github.com/junegunn/fzf/blob/master/install
    ARCH="$(uname -m)"
    KERNEL_OS="$(uname -s)"
    OS="$(uname -o 2> /dev/null || true)"
    local target=
    ASSET_NAME=
    case "$KERNEL_OS" in
        Darwin)
            case "$ARCH" in
                x86_64)  target="x86_64-apple-darwin"  ;;
                arm64)   target="aarch64-apple-darwin" ;;
                aarch64) target="aarch64-apple-darwin" ;;
            esac
            if [ -n "$target" ]; then
                ASSET_NAME="jieba_vim_rs-$target-$BINDING.dylib"
            fi
            DEST_NAME=jieba_vim_rs.so
            lib_ext=dylib
            ;;
        Linux)
            local libc=
            if has getconf && getconf GNU_LIBC_VERSION > /dev/null 2>&1; then
                libc=gnu
            fi
            if [ -n "$libc" ]; then
                case "$ARCH $OS" in
                    aarch64\ Android) target=                              ;;
                    aarch64*)         target="aarch64-unknown-linux-$libc" ;;
                    x86_64*)          target="x86_64-unknown-linux-$libc"  ;;
                    amd64*)           target="x86_64-unknown-linux-$libc"  ;;
                esac
            fi
            if [ -n "$target" ]; then
                ASSET_NAME="jieba_vim_rs-$target-$BINDING.so"
            fi
            DEST_NAME=jieba_vim_rs.so
            lib_ext=so
            ;;
        CYGWIN | MINGW | MSYS | Windows)
            target="x86_64-pc-windows-msvc"
            ASSET_NAME=jieba_vim_rs-$target-$BINDING.dll
            case "$BINDING" in
                py3)   DEST_NAME=jieba_vim_rs.pyd ;;
                lua51) DEST_NAME=jieba_vim_rs.dll ;;
            esac
            lib_ext=dll
    esac
    LIB_NAME=$lib_stem.$lib_ext
}

download_release() {
    if [ -z "$ASSET_NAME" ]; then
        return 1
    fi
    local curr_commit="$(git rev-parse HEAD)"
    local curr_tag="$(git tag --points-at "$curr_commit" 2> /dev/null)"
    if [ -z "$curr_tag" ]; then
        return 1
    fi
    local url="https://github.com/kkew3/jieba.vim/releases/download/$curr_tag/$ASSET_NAME"
    curl -fsSL -o "$DEST_DIR/$DEST_NAME" "$url"
}

download_release_url() {
    if [ -z "$ASSET_NAME" ]; then
        echo "jieba.vim build: unsupported platform for cdylib download: $ARCH $KERNEL_OS $OS" >&2
        return 1
    fi
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
