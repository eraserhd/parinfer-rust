#!/bin/bash

result=0
for VIM_TO_TEST in /usr/local/bin/vim /usr/local/Cellar/neovim/HEAD-0f1bc5d_1/bin/nvim
do
  export VIM_TO_TEST
  echo " === $VIM_TO_TEST ==="
  vim --clean -u tests/run.vim || result=$?
done

exit $result
