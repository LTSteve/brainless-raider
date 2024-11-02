del /q ".\out\*"
for /D %%p IN (".\out\*.*") DO rmdir "%%p" /s /q

cargo build --release --target wasm32-unknown-unknown

wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/brainless_raider.wasm

echo F|xcopy ".\web\index.html" ".\out\index.html" /a
xcopy ".\res\*" ".\out\res" /h /i /c /k /e /r /y

7z a -r index.zip ./out/*