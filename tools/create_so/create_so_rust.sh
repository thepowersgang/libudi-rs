#!/bin/sh
set -eu

if [ $# -ne 1 ]; then
  echo "Usage: $0 <libfile>"
  exit 1
fi

shortname=$(basename $1)
shortname=${shortname%%.a}
shortname=${shortname##libudi_}

ld -shared -o $(dirname $0)/libudi.so $(dirname $0)/libudi.ld
ld -shared -o $shortname.so $1 $(dirname $0)/libudi.so -g -T $(dirname $0)/link.ld --retain-symbols-file=$(dirname $0)/create_so_retain.txt --no-undefined

OUTFILE=$(realpath $shortname.so)
cd $(dirname $0)/../fix_elf
cargo run --quiet -- ${OUTFILE}
