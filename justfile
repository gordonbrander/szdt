default:
  just --list

# Test workspace
test:
    cd rust && cargo test --workspace

# Lint
clippy:
    cd rust && cargo clippy --all-targets --all-features -- -D warnings

# Check code formatting
check_fmt:
    cd rust && cargo fmt --all -- --check

# Format code
fmt:
    cd rust && cargo fmt --all

# Build and install CLI
install_cli:
    cargo install --path rust/szdt_cli

# Build WASM bindings for web
build_szdt_web:
    cd rust/szdt_wasm && wasm-pack build --target web --out-dir pkg/web

# Build WASM bindings for web
build_szdt_node:
    cd rust/szdt_wasm && wasm-pack build --target node --out-dir pkg/node

# Vend WASM files to docs website
vend_wasm: build_szdt_web
	mkdir -p "docs/static"
	cp -a rust/szdt_wasm/pkg/web/* "website/vendor"
	@echo "Copied szdt_wasm artifacts to static"

# Build website dev
build_website_dev: vend_wasm
	cd website && npm run build:dev

# Build website prod
build_website_prod: vend_wasm
	cd website && npm run build:prod

serve_website:
	cd website && npm run serve

clean:
    rm -rf "docs"
