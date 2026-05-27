import "debug.just"
import "quality.just"
import "rust.just"
import "test.just"
import "wasm.just"

default:
    @just --list

build: rust-build wasm-build

check: rust-check wasm-check

debug: debug-build

quality-check: quality

test: test-ci

ci: quality rust-ci wasm-ci test-nextest
