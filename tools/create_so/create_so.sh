#!/bin/sh
set -eu

SRCDIR=$1
ABI=$2
UDIPROPS=$SRCDIR/udiprops.txt

cat ${UDIPROPS} | sed 's/ *#.*//' | grep -v '^$' | tr '\t' ' ' | grep -v 'source_files\|source_requires\|compile_options' | tr '\n' '\0' > .udiprops.bin

shortname=`grep 'shortname ' ${UDIPROPS} | head -n 1 | awk '{print $2}'`
firstmodule=`grep 'module ' ${UDIPROPS} | head -n 1 | awk '{print $2}'`

case $ABI in
ia32|amd64)
    objcopy $SRCDIR/bin/$ABI/$firstmodule --add-section .udiprops=.udiprops.bin .primodule
    ;;
*)
    echo "Unknown architecture \"${ABI}\""
    exit 1
    ;;
esac
rm .udiprops.bin

ld -shared -o $(dirname $0)/libudi.so $(dirname $0)/libudi.ld
ld -shared -o $shortname.so .primodule $(dirname $0)/libudi.so -g -T $(dirname $0)/link.ld --retain-symbols-file=$(dirname $0)/create_so_retain.txt --no-undefined
rm .primodule 

OUTFILE=$(realpath $shortname.so)
cd $(dirname $0)/../fix_elf
cargo run -- ${OUTFILE}
