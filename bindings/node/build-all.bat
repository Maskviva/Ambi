@echo off
echo ===================================================
echo   Ambi Node.js Bindings - Custom Matrix Builder
echo ===================================================

echo.
echo [0/5] Preparing Node modules on Host...
call npm install

echo.
echo[1/5] Building Custom Docker Image (ambi-builder)...
docker build -t ambi-builder ./builder

echo.
echo [2/5] Building Windows x64 MSVC...
call npx napi build --platform --release --target x86_64-pc-windows-msvc

echo.
echo [3/5] Building Linux x64 GNU via Custom Docker...
docker run --rm -e RUSTC_WRAPPER="" -e CARGO_BUILD_RUSTC_WRAPPER="" -v "%cd%\..\..:/workspace" -w /workspace/bindings/node ambi-builder napi build --target x86_64-unknown-linux-gnu --release

echo.
echo[4/5] Building Linux ARM64 GNU via Custom Docker...
docker run --rm -e RUSTC_WRAPPER="" -e CARGO_BUILD_RUSTC_WRAPPER="" -v "%cd%\..\..:/workspace" -w /workspace/bindings/node ambi-builder napi build --target aarch64-unknown-linux-gnu --release

echo.
echo [5/5] Building Linux ARM64 MUSL Alpine via Custom Docker...
docker run --rm -e RUSTC_WRAPPER="" -e CARGO_BUILD_RUSTC_WRAPPER="" -v "%cd%\..\..:/workspace" -w /workspace/bindings/node ambi-builder napi build --target aarch64-unknown-linux-musl --release

echo.
echo ===================================================
echo All binaries compiled! Moving to npm folders...
echo ===================================================

call npx napi prepublish -t npm

node -e "const fs=require('fs');const p='lib/index.d.ts';let c=fs.readFileSync(p,'utf8');if(!c.includes('type JsonValue')){fs.writeFileSync(p,'type JsonValue = any;\n'+c)}"

echo.
echo Build and Distribution Complete!
pause