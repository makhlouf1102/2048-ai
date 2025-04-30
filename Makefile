# Variables
BUILD_DIR := build
EXECUTABLE := cpp_2048

# Default target: configure and build
all:
	@mkdir -p $(BUILD_DIR)
	cd $(BUILD_DIR) && cmake ..
	cd $(BUILD_DIR) && make
	

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)

# Run the executable
run:
	./$(BUILD_DIR)/$(EXECUTABLE)


build: clean all

build_run: build run


