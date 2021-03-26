#!/bin/bash
export RUST_BACKTRACE=full

# make a file of random data
head -c 200M </dev/urandom >randomFile
cargo build

# encrypt from stdin to stdout, decrypt from stdin to file
cat randomFile | target/debug/cloaker_cli -E -O -p "00000000000" | target/debug/cloaker_cli -D -o decryptedFile -p "00000000000"
if diff randomFile decryptedFile
    then echo "files match"
    else echo "files differ"
fi

rm randomFile decryptedFile


# ./target/debug/cloaker_cli -E -p "00000000000" <randomFile
# ./target/debug/cloaker_cli -D -o decryptedFile -p "00000000000" < encrypted.cloaker

# target/debug/cloaker_cli -e randomFile -p "00000000000" -O > encrypted.cloaker
# target/debug/cloaker_cli -d encrypted.cloaker -p "00000000000" -o decryptedFile

# rm randomFile encrypted.cloaker decryptedFile