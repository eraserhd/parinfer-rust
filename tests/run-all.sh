#!/bin/bash

set -e

node_version=$(node --version)
node_version=${node_version#v}
node_version=${node_version%%.*}
if [[ $node_version -lt 8 ]]
then
  printf 'bad version of node (does not support WebAssembly)\n' >&2
fi

( cargo build --release
  cargo test
  cargo +nightly build
  cargo +nightly web test --nodejs )

result=0
for VIM_TO_TEST in /usr/local/bin/vim /usr/local/Cellar/neovim/HEAD-0f1bc5d_1/bin/nvim
do
  export VIM_TO_TEST
  echo " === $VIM_TO_TEST ==="
  vim --clean -u tests/run.vim || result=$?
done

exit $result
