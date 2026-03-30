default:
    @just --list

dev:
    pnpm tauri dev

build:
    pnpm tauri build

lint:
    pnpm eslint src/
    cd src-tauri && cargo clippy -- -D warnings

format:
    pnpm prettier --write "src/**/*.{ts,vue,css,json}"
    cd src-tauri && cargo fmt

check:
    cd src-tauri && cargo check

clean:
    cd src-tauri && cargo clean
    rm -rf dist node_modules/.vite
