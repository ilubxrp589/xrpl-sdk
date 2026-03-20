#!/usr/bin/env bash
set -e

echo "Building xrpl-core WASM package..."
wasm-pack build crates/xrpl-core \
    --target web \
    --out-dir ../../pkg/xrpl-core \
    --features wasm

echo ""
echo "WASM build complete. Output in pkg/"
echo "xrpl-core:"
ls -la pkg/xrpl-core/*.wasm 2>/dev/null || true
echo ""
echo "Exported functions:"
grep "export function" pkg/xrpl-core/xrpl_core.d.ts 2>/dev/null || true
