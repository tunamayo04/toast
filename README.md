![alt tet](resources/logo.png)

# Toast
A small hobby operating system written in Rust

## Current state
- Limine bootloader and boot protocol
- Memory Allocation
- Async/Await
- Keyboard
- Disk I/O
- ext2 file system 

## Dependencies
- QEMU

## Installation
- Create a disk image
    - `qemu-img create -f raw build/toast-disk.img 1G`
- Partition disk image
    - `fdisk toast-disk.img`
- Format disk image
    - `losetup --partscan --show --find toast-disk.img`
    - `mkfs.ext2 /dev/loop7`
- Run
    - `make run`
