# Silk VS Code extension

This extension adds basic language support for the Silk programming language in VS Code.

## Included

- Syntax highlighting for `.silk` files
- Comment and bracket auto-completion
- Basic snippets for common Silk constructs

## Install locally

1. Open the extension folder in VS Code.
2. Press `F5` to run the extension host.
3. Or package it for local installation using VSCE.

## Example

```silk
func greet(name) {
    return "Hello, " + name
}

if (true) {
    print(greet("Silk"))
}
```

## Notes

This is a lightweight starter extension. It provides editor support for Silk syntax and can be extended with semantic features later, such as linting, formatting, or a full language server.
