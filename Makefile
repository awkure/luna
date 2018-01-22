# TODO : detect grub2-mkrescue dependent on linux distro
include Config.mk

ISO    := build/luna-$(ARCH).iso
KERNEL := build/kernel-$(ARCH).bin
TARGET ?= luna-$(ARCH)

linker_ld 		:= platform/$(ARCH)/linker.ld
grub_cfg  		:= platform/$(ARCH)/grub.cfg

assembly_source := $(wildcard platform/$(ARCH)/*.S)
assembly_object := $(patsubst platform/$(ARCH)/%.S, build/arch/$(ARCH)/%.o, $(assembly_source))

rust_kernel     := target/$(TARGET)/release/libluna.a 

rescue_path 	:= build/isofiles

.DEFAULT_GOAL := check # help

repeat : clean iso run

check :
	xargo check --target $(TARGET)

all : $(KERNEL)

clean :
	xargo clean
	$(RM) -r build *~ 

distclean : clean

# compile assembly files
build/arch/$(ARCH)/%.o : platform/$(ARCH)/%.S
	mkdir -p $(shell dirname $@)
	nasm -felf64 $< -o $@

run :
	@qemu-system-x86_64 -cdrom $(ISO) -s

debug :
	@qemu-system-x86_64 -cdrom $(ISO) -s -S 

gdb :
	gdb "build/kernel-x86_64.bin" -ex "target remote :1234"

iso : $(ISO)

$(ISO) : $(KERNEL) # $(grub_cfg)
	mkdir -p $(rescue_path)/boot/grub
	cp $(KERNEL) $(rescue_path)/boot/kernel.bin
	cp $(grub_cfg) $(rescue_path)/boot/grub
	grub-mkrescue -o $(ISO) $(rescue_path)
	rm -r $(rescue_path)

$(KERNEL) : kernel $(rust_kernel) $(assembly_object) $(linker_ld)
	ld -n --gc-sections -T $(linker_ld) -o $(KERNEL) $(assembly_object) $(rust_kernel) -m elf_x86_64 

kernel :
	xargo build --target $(TARGET) --release

help :
	@- echo "make"
	@- echo "    [ all | iso | kernel | run | debug | clean | distclean ]"
	@- echo -e "    ?ARCH=[ x86_64 | amd64 ]\n"
	@- echo "example:"
	@- echo "    make iso ?ARCH=x86_64"

.PHONY : all clean distclean run iso help kernel no_targets__ list
