#!/bin/sh
set -eu

if [ $# -ne 1 ]; then
  echo "Usage: $0 <libfile>"
  exit 1
fi

shortname=$(basename $1)
shortname=${shortname%%.a}
shortname=${shortname##libudi_}

# re-generate libudi.so
ld -shared -o $(dirname $0)/libudi.so $(dirname $0)/libudi.ld

# Link the executable
LD_ARGS="-shared -o $shortname.so -u udi_init_info $1"
LD_ARGS=$LD_ARGS" -u libudi_rs_udiprops"  # Hacky name in udiprops parser
LD_ARGS=$LD_ARGS" --gc-sections"
LD_ARGS=$LD_ARGS" -g"
LD_ARGS=$LD_ARGS" -T $(dirname $0)/link.ld"
#LD_ARGS=$LD_ARGS" $(dirname $0)/libudi.so"
#LD_ARGS=$LD_ARGS" --no-undefined"
LD_ARGS=$LD_ARGS" --retain-symbols-file=$(dirname $0)/create_so_retain.txt"
#LD_ARGS=$LD_ARGS" -Map $shortname.map"
ld $LD_ARGS
    

OUTFILE=$(realpath $shortname.so)
cd $(dirname $0)/../fix_elf
cargo run --quiet -- ${OUTFILE}
