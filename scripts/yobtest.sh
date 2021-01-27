#!/bin/sh
set -e

cd "$(dirname $0)/../"
mkdir -p yobtest
cd yobtest

# TODO: Install c-chess-cli from https://github.com/lucasart/c-chess-cli

# Output looks like this
# Score of Yobmef vs Sunfish: 0 - 3 - 0  [0.000] 3
# SPRT: LLR = 0.000 [-2.944,2.944]
# I think we wait for LLR to be equal to one of the endpoints, if it is
# equal to 2.944, we reject the null hypothesis. etc. my stats isn't that good yet D:!

# Book from https://github.com/official-stockfish/books/
BOOK=noob_3moves.epd
 
if ! [ -f $BOOK ]; then
  curl -LO https://raw.githubusercontent.com/official-stockfish/books/master/$BOOK.zip  
  # For some reason the zipfile is in some retarded file structure, we extract to
  # stdout and redirect to avoid this. (there should only be one file)
  unzip -p $BOOK.zip > $BOOK
fi

# Copy the latest yobmef so we don't get fucked on recompile.
mkdir -p /tmp/yobmef
cp ../target/release/yobmef /tmp/yobmef/latest
 
c-chess-cli \
  -engine cmd=/tmp/yobmef/latest name='Yobmef DEV' \
  -engine cmd=$1 \
  -each tc=10 -games 100 \
  -openings file=$BOOK \
  -concurrency 2 \
  -resign 3 700 -draw 8 10 -log \

