.PHONY: all build clean test robot remote-controller

all: robot remote-controller

robot:
	@echo "Flashing robot..."
	cd robot && cargo run

remote-controller:
	@echo "Flashing remote-controller..."
	cd remote-controller && cargo run

clean:
	@echo "Cleaning robot..."
	cd robot && cargo clean
	@echo "Cleaning remote-controller..."
	cd remote-controller && cargo clean

check:
	@echo "Checking robot..."
	cd robot && cargo check
	@echo "Checking remote-controller..."
	cd remote-controller && cargo check
