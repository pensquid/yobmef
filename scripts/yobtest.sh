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
 
c-chess-cli \
  -engine cmd=yobmef \
  -engine cmd=$1 \
  -each tc=10+0.1 -games 10 \
  -openings file=$BOOK \
  -sprt elo0=0 elo1=5 \
  -resign 3 700 -draw 8 10 -log \

