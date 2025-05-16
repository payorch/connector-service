.PHONY: all format generate clean build lint

# Default target
all: format generate

# Format proto files
format:
	@echo "Formatting proto files..."
	buf format -w

# Generate code from proto files (e.g., OpenAPI specs)
generate:
	@echo "Generating code from proto files..."
	buf generate

# Validate proto files
# This can catch issues before generating code or compiling
build:
	@echo "Building/validating proto files..."
	buf build

# Lint proto files
lint:
	@echo "Linting proto files..."
	buf lint

# Clean generated files
clean:
	@echo "Cleaning generated files..."
	rm -rf gen
