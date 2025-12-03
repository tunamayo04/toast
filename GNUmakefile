# Nuke built-in rules and variables.
override MAKEFLAGS += -rR

override IMAGE_NAME := toast
override DISK_IMG := toast-disk
override CPU_MODEL := Nehalem-v2

# Convenience macro to reliably declare user overridable variables.
define DEFAULT_VAR =
    ifeq ($(origin $1),default)
        override $(1) := $(2)
    endif
    ifeq ($(origin $1),undefined)
        override $(1) := $(2)
    endif
endef

# Toolchain for building the 'limine' executable for the host.
override DEFAULT_HOST_CC := cc
$(eval $(call DEFAULT_VAR,HOST_CC,$(DEFAULT_HOST_CC)))
override DEFAULT_HOST_CFLAGS := -g -O2 -pipe
$(eval $(call DEFAULT_VAR,HOST_CFLAGS,$(DEFAULT_HOST_CFLAGS)))
override DEFAULT_HOST_CPPFLAGS :=
$(eval $(call DEFAULT_VAR,HOST_CPPFLAGS,$(DEFAULT_HOST_CPPFLAGS)))
override DEFAULT_HOST_LDFLAGS :=
$(eval $(call DEFAULT_VAR,HOST_LDFLAGS,$(DEFAULT_HOST_LDFLAGS)))
override DEFAULT_HOST_LIBS :=
$(eval $(call DEFAULT_VAR,HOST_LIBS,$(DEFAULT_HOST_LIBS)))

qemu_flags := -s \
			  -cpu $(CPU_MODEL) \
			  -cdrom $(IMAGE_NAME).iso \
			  -device ahci,id=ahci \
			  -serial stdio \
			  #-drive id=disk,file=$(DISK_IMG).img,if=none \
			  #-device ide-hd,drive=disk,bus=ahci.0


.PHONY: all all-hdd run run-uefi run-hdd run-hdd-uefi kernel clean distclean

all: $(IMAGE_NAME).iso

all-hdd: $(IMAGE_NAME).hdd

run: $(IMAGE_NAME).iso
	@qemu-system-x86_64 $(qemu_flags) -m 4G -no-reboot

run-tests: $(IMAGE_NAME).iso-test
	@qemu-system-x86_64 $(qemu_flags) -m 4G -no-reboot -device isa-debug-exit,iobase=0xf4,iosize=0x04 -display none || [ $$? -eq 33 ]
	@exit 0

run-with-log: $(IMAGE_NAME).iso
	@qemu-system-x86_64 $(qemu_flags) -d int -no-reboot

debug: $(IMAGE_NAME).iso
	@qemu-system-x86_64 $(qemu_flags) -S -m 4G

gdb:
	@gdb kernel/kernel -ex "target remote :1234"

run-uefi: ovmf $(IMAGE_NAME).iso
	@qemu-system-x86_64 $(qemu_flags) -M q35 -m 2G -bios ovmf/OVMF.fd -boot d

run-hdd: $(IMAGE_NAME).hdd
	@qemu-system-x86_64 -M q35 -m 2G -hda $(IMAGE_NAME).hdd

run-hdd-uefi: ovmf $(IMAGE_NAME).hdd
	@qemu-system-x86_64 -M q35 -m 2G -bios ovmf/OVMF.fd -hda $(IMAGE_NAME).hdd

ovmf:
	@mkdir -p ovmf
	@cd ovmf && curl -Lo OVMF.fd https://retrage.github.io/edk2-nightly/bin/RELEASEX64_OVMF.fd

limine:
	@git clone https://github.com/limine-bootloader/limine.git --branch=v7.x-binary --depth=1
	@$(MAKE) -C limine \
		CC="$(HOST_CC)" \
		CFLAGS="$(HOST_CFLAGS)" \
		CPPFLAGS="$(HOST_CPPFLAGS)" \
		LDFLAGS="$(HOST_LDFLAGS)" \
		LIBS="$(HOST_LIBS)"

kernel:
	@$(MAKE) -C kernel

kernel-test:
	@$(MAKE) -C kernel test

$(IMAGE_NAME).iso: limine kernel
	@rm -rf iso_root
	@mkdir -p iso_root/boot
	@cp -v kernel/kernel iso_root/boot/
	@mkdir -p iso_root/boot/limine
	@cp -v limine.cfg limine/limine-bios.sys limine/limine-bios-cd.bin limine/limine-uefi-cd.bin iso_root/boot/limine/
	@mkdir -p iso_root/EFI/BOOT
	@cp -v limine/BOOTX64.EFI iso_root/EFI/BOOT/
	@cp -v limine/BOOTIA32.EFI iso_root/EFI/BOOT/
	@xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o $(IMAGE_NAME).iso 2> /dev/null
	@./limine/limine bios-install $(IMAGE_NAME).iso 2> /dev/null
	@rm -rf iso_root

$(IMAGE_NAME).iso-test: limine kernel-test
	@rm -rf iso_root
	@mkdir -p iso_root/boot
	@cp -v kernel/kernel iso_root/boot/
	@mkdir -p iso_root/boot/limine
	@cp -v limine.cfg limine/limine-bios.sys limine/limine-bios-cd.bin limine/limine-uefi-cd.bin iso_root/boot/limine/
	@mkdir -p iso_root/EFI/BOOT
	@cp -v limine/BOOTX64.EFI iso_root/EFI/BOOT/
	@cp -v limine/BOOTIA32.EFI iso_root/EFI/BOOT/
	@xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o $(IMAGE_NAME).iso 2> /dev/null
	@./limine/limine bios-install $(IMAGE_NAME).iso 2> /dev/null
	@rm -rf iso_root

$(IMAGE_NAME).hdd: limine kernel
	@rm -f $(IMAGE_NAME).hdd
	@dd if=/dev/zero bs=1M count=0 seek=64 of=$(IMAGE_NAME).hdd
	@sgdisk $(IMAGE_NAME).hdd -n 1:2048 -t 1:ef00
	@./limine/limine bios-install $(IMAGE_NAME).hdd
	@mformat -i $(IMAGE_NAME).hdd@@1M
	@mmd -i $(IMAGE_NAME).hdd@@1M ::/EFI ::/EFI/BOOT ::/boot ::/boot/limine
	@mcopy -i $(IMAGE_NAME).hdd@@1M kernel/kernel ::/boot
	@mcopy -i $(IMAGE_NAME).hdd@@1M limine.cfg limine/limine-bios.sys ::/boot/limine
	@mcopy -i $(IMAGE_NAME).hdd@@1M limine/BOOTX64.EFI ::/EFI/BOOT
	@mcopy -i $(IMAGE_NAME).hdd@@1M limine/BOOTIA32.EFI ::/EFI/BOOT

clean:
	@rm -rf iso_root $(IMAGE_NAME).iso $(IMAGE_NAME).hdd
	@$(MAKE) -C kernel clean

distclean: clean
	@rm -rf limine ovmf
	@$(MAKE) -C kernel distclean
