# Run on the father folder of arceos
# Because of unknown reason, this file only run by copy out to bash
cd hello_app
cargo build --target riscv64gc-unknown-none-elf --release
rust-objcopy --binary-architecture=riscv64 --strip-all -O binary target/riscv64gc-unknown-none-elf/release/hello_app ./hello_app.bin
cd ..

cd hello_app2
cargo build --target riscv64gc-unknown-none-elf --release
rust-objcopy --binary-architecture=riscv64 --strip-all -O binary target/riscv64gc-unknown-none-elf/release/hello_app2 ./hello_app2.bin
cd ..

dd if=/dev/zero of=./apps.bin bs=1M count=32

app_num=2
printf "$(printf '%02x' $app_num)" | xxd -r -p | dd of=apps.bin conv=notrunc bs=1 seek=0

app_size=$(stat -c %s ./hello_app/hello_app.bin)
printf "$(printf '%04x' $app_size)" | xxd -r -p | dd of=./apps.bin conv=notrunc bs=1 seek=1
dd if=./hello_app/hello_app.bin of=./apps.bin conv=notrunc bs=1 seek=3

app_size2=$(stat -c %s ./hello_app2/hello_app2.bin)
printf "$(printf '%04x' $app_size2)" | xxd -r -p | dd of=./apps.bin conv=notrunc bs=1 seek=9
dd if=./hello_app2/hello_app2.bin of=./apps.bin conv=notrunc bs=1 seek=11

mkdir -p ./arceos/payload
mv ./apps.bin ./arceos/payload/apps.bin
