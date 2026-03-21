#!/usr/bin/env bash

# Environment variables:
#
# If JIEBA_VIM_BUILD_FROM_SOURCE=1, then skip downloading cdylib and build from
# source directly.
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

download_release() {
    if [ "$JIEBA_VIM_BUILD_FROM_SOURCE" = "1" ]; then
        return 2
    fi

    curr_commit="$(git rev-parse HEAD)"
    curr_tag="$(git tag --points-at "$curr_commit" 2> /dev/null)"
    if [ -z "$curr_tag" ]; then
        return 1
    fi
    local binding=
    local dest_dir=
    if [ "$JIEBA_VIM_INSTALL_NVIM" = "1" ]; then
        binding=lua51
        dest_dir=lua/jieba_vim
    else
        binding=py3
        dest_dir=pythonx/jieba_vim
    fi
    arch="$(uname -m)"
    os="$(uname -s)"
    url=""
    case "$arch-$os" in
        x86_64-Darwin)
            url=https://github.com/kkew3/jieba.vim/releases/download/${curr_tag}/jieba_vim_rs-x86_64-apple-darwin-$binding.dylib
            name=jieba_vim_rs.so
            ;;
        aarch64-Darwin | arm64-Darwin)
            url=https://github.com/kkew3/jieba.vim/releases/download/${curr_tag}/jieba_vim_rs-aarch64-apple-darwin-$binding.dylib
            name=jieba_vim_rs.so
            ;;
        x86_64-Linux | amd64-Linux)
            url=https://github.com/kkew3/jieba.vim/releases/download/${curr_tag}/jieba_vim_rs-x86_64-unknown-linux-gnu-$binding.so
            name=jieba_vim_rs.so
            ;;
        aarch64-Linux | arm64-Linux)
            url=https://github.com/kkew3/jieba.vim/releases/download/${curr_tag}/jieba_vim_rs-aarch64-unknown-linux-gnu-$binding.so
            name=jieba_vim_rs.so
            ;;
    esac
    if [ -z "$url" ]; then
        return 1
    fi
    curl -fsSLo "$dest_dir/$name" "$url"
}

build_from_source() {
    local color_when=
    if [ -n "$VIMRUNTIME" ]; then
        color_when=never
    else
        color_when=auto
    fi
    local binding=
    local lib_stem=
    local dest_dir=
    if [ "$JIEBA_VIM_INSTALL_NVIM" = "1" ]; then
        binding=lua51
        lib_stem=libjieba_vim_jieba_vim_rs
        dest_dir=lua/jieba_vim
    else
        binding=py3
        lib_stem=libjieba_vim_rs
        dest_dir=pythonx/jieba_vim
    fi
    local lib_ext=
    case "$(uname -s)" in
        Darwin)
            lib_ext=dylib
            ;;
        *)
            # Assume that build.sh is never run on Windows.
            lib_ext=so
            ;;
    esac
    cdylib_name=$lib_stem.$lib_ext
    dest_name=jieba_vim_rs.so
    # rm: used to delete $dest_name in case it's a symlink
    rm -f $dest_dir/$dest_name
    cargo clean --color=$color_when --manifest-path rust_backend/Cargo.toml
    cargo build -r --color=$color_when \
        --manifest-path rust_backend/Cargo.toml \
        --package jieba_vim_rs_binding_$binding \
        && cp rust_backend/target/release/$cdylib_name $dest_dir/$dest_name
}

if has git uname curl; then
    if download_release; then
        exit 0
    fi
fi
build_from_source
