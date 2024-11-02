At this point I switched to windows because I was running into issues on NIX

Install 7z (windows requirement, alternatively, just do the webbuild.cmd stuff by hand)

Install wasm-server-runner

```
cargo install wasm-server-runner
```

Set up cargo to use it

(See .cargo/config.toml)

Debug

```
cargo run --target wasm32-unknown-unknown
```

Needed to update wasm-bindgen (this could be different based on what 'debug' gives at the time of running)

```
cargo update -p wasm-bindgen --precise 0.2.95
```

Before building, wasm-bindgen needs to be installed:

```
cargo install wasm-bindgen-cli
```

To create a build, run:

```
webbuild.cmd
```

This will copy the web build to ./out, add ./web/index.html and ./res, and compress the contents of ./out to index.zip