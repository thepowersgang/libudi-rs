#!/bin/sh
# Complete linking of a pre-compiled non-rust UDI binary for use with the example environment
# cspell:ignore primodule libudi
set -eu

if [ $# -ne 2 ]; then
  echo "Usage: $0 <srcdir> <abi>"
  exit 1
fi

SRCDIR=$1
ABI=$2
UDIPROPS=$SRCDIR/udiprops.txt

# Clean up `udiprops.txt` and convert into a flattened binary form
cat ${UDIPROPS} | sed 's/ *#.*//' | grep -v '^$' | tr '\t' ' ' | grep -v 'source_files\|source_requires\|compile_options' | tr '\n' '\0' > .udiprops.bin

shortname=`grep '^shortname ' ${UDIPROPS} | head -n 1 | awk '{print $2}'`
first_module=`grep '^module ' ${UDIPROPS} | head -n 1 | awk '{print $2}'`

TMP_PRIMODULE=.primodule_${shortname}.o

case $ABI in
ia32|amd64)
    objcopy $SRCDIR/bin/$ABI/$first_module --add-section .udiprops=.udiprops.bin ${TMP_PRIMODULE}
    ;;
*)
    echo "Unknown architecture \"${ABI}\""
    exit 1
    ;;
esac
rm .udiprops.bin

# Ensure that the stub `libudi.so` is generated
ld -shared -o $(dirname $0)/libudi.so $(dirname $0)/libudi.ld
ld -shared -o $shortname.so ${TMP_PRIMODULE} $(dirname $0)/libudi.so -g -T $(dirname $0)/link.ld --retain-symbols-file=$(dirname $0)/create_so_retain.txt --no-undefined
rm ${TMP_PRIMODULE}

# Apply fixes to the ELF file, so it can reference within the executable
OUTFILE=$(realpath $shortname.so)
cd $(dirname $0)/../fix_elf
cargo run -- ${OUTFILE}
