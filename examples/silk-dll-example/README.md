# Silk DLL example

This crate builds a tiny shared library that registers a module called `example_dll` into the global Silk VM.

## Build

```bash
cargo build --manifest-path examples/silk-dll-example/Cargo.toml
```

On Linux the output will be a `.so` file; on Windows it will be a `.dll` file.

## Use it in Silk

Load the shared library and call `silk_load_module()`, then import the registered module:

```silk
import "example_dll"

print(hello)
print(answer)
print(ready)
```

The module exports:

- `hello` => "hello from example dll"
- `answer` => 42
- `ready` => true
