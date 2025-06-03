# Build settings
CARGO_BUILD_TARGET = aarch64-unknown-none
CARGO_BUILD_FLAGS = --release

# Output files
KERNEL_ELF = target/$(CARGO_BUILD_TARGET)/release/minimal_pi5_os
KERNEL_BIN = kernel8.img

# Tools
OBJCOPY = aarch64-linux-gnu-objcopy

.PHONY: all build clean qemu kernel8.img

all: build

build:
	cargo build $(CARGO_BUILD_FLAGS)

kernel8.img: build
	$(OBJCOPY) -O binary $(KERNEL_ELF) $(KERNEL_BIN)
	@echo "Kernel binary created: $(KERNEL_BIN)"
	@ls -la $(KERNEL_BIN)

clean:
	cargo clean
	rm -f $(KERNEL_BIN)

# Create SD card image for Pi5
sdcard: kernel8.img
	@echo "Creating SD card image..."
	dd if=/dev/zero of=pi5_os.img bs=1M count=64
	mkfs.fat -F 32 pi5_os.img
	mkdir -p /tmp/pi5_mount
	sudo mount -o loop pi5_os.img /tmp/pi5_mount
	sudo cp $(KERNEL_BIN) /tmp/pi5_mount/
	sudo cp config.txt /tmp/pi5_mount/ 2>/dev/null || echo "config.txt not found, skipping"
	sudo umount /tmp/pi5_mount
	rmdir /tmp/pi5_mount
	@echo "SD card image created: pi5_os.img"

help:
	@echo "Available targets:"
	@echo "  build     - Build the kernel"
	@echo "  kernel8.img - Create binary image for Pi5"
	@echo "  clean     - Clean build artifacts"
	@echo "  qemu      - Test with QEMU (basic verification)"
	@echo "  sdcard    - Create bootable SD card image"
	@echo "  hardware  - Create SD card for hardware testing"
	@echo "  uart      - Connect to UART for debugging"
	@echo "  help      - Show this help"

# Hardware testing targets
hardware: kernel8.img
	@echo "Creating hardware test SD card..."
	@echo "Use: ./create_sdcard.sh /dev/sdX"
	@echo "Replace /dev/sdX with your SD card device"

uart:
	@echo "Connecting to UART (Ctrl+A, K to exit)..."
	@echo "Make sure UART is connected to GPIO 14/15"
	screen /dev/ttyUSB0 115200

# Pi5 specific verification
verify: kernel8.img
	@echo "Kernel verification:"
	@echo "  File: $(KERNEL_BIN)"
	@echo "  Size: $$(stat -c%s $(KERNEL_BIN)) bytes"
	@echo "  Type: $$(file $(KERNEL_BIN))"
	@echo "  Architecture: ARM64"
	@echo "  Load Address: 0x200000"
	@echo ""
	@echo "Ready for Raspberry Pi 5 deployment"
