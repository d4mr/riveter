# riveter

A command-line utility written in Rust to "rivet" together the structure and content of a project directory. It generates a combined overview suitable for providing context to Large Language Models (LLMs) or for documentation purposes.

## Motivation

When working with LLMs, providing sufficient context about a codebase or project structure is crucial for getting relevant and accurate responses. Manually copying directory trees and file contents is tedious and error-prone. This tool automates the process, creating a single output (either text or XML) that represents the project's layout and the content of its readable files.

## Features

- **Directory Tree:** Generates a visual representation of the directory structure.
- **File Content Concatenation:** Reads and includes the content of text files.
- **Output Formats:** Supports plain text (`text`) and structured XML (`xml`) output.
- **.gitignore Integration:** Automatically respects `.gitignore` rules found within the project to exclude irrelevant files/directories (can be disabled).
- **Custom Exclusions:** Allows specifying additional ignore patterns (using gitignore syntax) via the command line.
- **Depth Control:** Limits the directory traversal depth.
- **Graceful File Handling:** Skips binary files or files with non-UTF8 content, issuing warnings instead of crashing.
- **Cross-Platform:** Built with Rust, runs on Linux, macOS, and Windows.

## Installation

### Prerequisites

- [Rust programming language and Cargo](https://www.rust-lang.org/tools/install) (latest stable version recommended).

### From Pre-Built Binaries

You can use [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) to install a pre-built binary for your platform.

```bash
cargo binstall riveter
```

### From Source

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/d4mr/riveter
    cd riveter
    ```
2.  **Build:**
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/riveter`.
3.  **(Optional) Install globally using Cargo:**
    From within the cloned directory:
    ```bash
    cargo install --path .
    ```
    This will install the binary into your Cargo bin directory (usually `~/.cargo/bin/`), making it available in your PATH.

## Usage

The tool is run from the command line.

```

riveter [OPTIONS]

```

By default, it processes the current directory (`.`) and outputs in text format to standard output.

> [!TIP]
> You can pipe the output to `pbcopy` on macOS to copy it to your clipboard
>
> ```bash
> riveter -f xml | pbcopy
> ```

### Options

Here's the output from `riveter --help`:

```text
Processes a directory, creating a structure view (text tree or XML) and concatenating readable file contents. Uses gitignore-style patterns for exclusion and optionally respects .gitignore files. Defaults to the current directory if none is specified.

Usage: riveter [OPTIONS]

Options:
  -d, --directory <DIRECTORY>
          The directory to process

          [default: .]

  -f, --format <FORMAT>
          Output format

          [default: text]
          [possible values: text, xml]

  -m, --max-depth <DEPTH>
          Maximum depth to traverse (0 means no limit)

          [default: 0]

  -x, --exclude [<PATTERN>...]
          Gitignore-style patterns to exclude files/directories. Applied *after* gitignore rules

      --respect-gitignore
          Respect .gitignore files found in the directory structure

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Key Arguments:

- `-d, --directory <DIR>`: Specify the target directory to process.
- `-f, --format <FORMAT>`: Choose the output format (`text` or `xml`).
- `-m, --max-depth <N>`: Limit directory traversal to `N` levels deep (0 = unlimited).
- `-x, --exclude <PATTERN>`: Provide one or more gitignore-style patterns to exclude specific files or directories (e.g., `-x "*.log" -x "build/"`). This is applied _in addition_ to `.gitignore` rules.
- `--respect-gitignore=false`: Disable the automatic use of `.gitignore` files.

## Examples

1.  **Process the current directory (default settings):**

    ```bash
    riveter
    ```

    (Outputs text format, respects `.gitignore`, unlimited depth)

2.  **Process a specific project directory and output as XML:**

    ```bash
    riveter -d ../my-other-project -f xml
    ```

3.  **Process the current directory, excluding log files and the `dist` folder:**

    ```bash
    riveter -x "*.log" -x "dist/"
    ```

4.  **Process the current directory, but only show the first 2 levels:**

    ```bash
    riveter -m 2
    ```

5.  **Process the current directory, withour respecting any `.gitignore` files:**

    ```bash
    riveter --respect-gitignore=false
    ```

6.  **Save the XML output to a file:**
    ```bash
    riveter -f xml > project_context.xml
    ```

## Output Formats

### Text Format (`-f text`)

Provides a human-readable output:

```text
--- Directory Tree ---
my_project/
  src/
    main.rs
    lib.rs
  tests/
    integration_test.rs
  .gitignore
  Cargo.toml
  README.md

--- File Contents ---
========================================
File: .gitignore
========================================
/target
Cargo.lock

========================================
File: Cargo.toml
========================================
[package]
name = "my_project"
# ... rest of Cargo.toml content ...

========================================
File: README.md
========================================
# My Project
This is a sample project.
# ... rest of README.md content ...

========================================
File: src/lib.rs
========================================
// Library code here
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

========================================
File: src/main.rs
========================================
// Main application code here
fn main() {
    println!("Hello, world!");
}

========================================
File: tests/integration_test.rs
========================================
// Integration tests here
#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

```

### XML Format (`-f xml`)

Provides a structured XML output, suitable for programmatic parsing:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<projectContext rootPath="/absolute/path/to/my_project">
  <tree>
    <dir name="my_project">
      <file name=".gitignore"/>
      <file name="Cargo.toml"/>
      <file name="README.md"/>
      <dir name="src">
        <file name="lib.rs"/>
        <file name="main.rs"/>
      </dir>
      <dir name="tests">
        <file name="integration_test.rs"/>
      </dir>
    </dir>
  </tree>
  <fileContents>
    <file path=".gitignore"><![CDATA[/target
Cargo.lock
]]></file>
    <file path="Cargo.toml"><![CDATA[[package]
name = "my_project"
# ... rest of Cargo.toml content ...
]]></file>
    <file path="README.md"><![CDATA[# My Project
This is a sample project.
# ... rest of README.md content ...
]]></file>
    <file path="src/lib.rs"><![CDATA[// Library code here
pub fn add(left: usize, right: usize) -> usize {
    left + right
}
]]></file>
    <file path="src/main.rs"><![CDATA[// Main application code here
fn main() {
    println!("Hello, world!");
}
]]></file>
    <file path="tests/integration_test.rs"><![CDATA[// Integration tests here
#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}
]]></file>
  </fileContents>
</projectContext>
```

_Note: File content is wrapped in `<![CDATA[...]]>` sections to handle special characters correctly._

## License

Licensed under Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Acknowledgements

This tool relies on several excellent Rust crates, including:

- [`clap`](https://crates.io/crates/clap) for command-line argument parsing.
- [`ignore`](https://crates.io/crates/ignore) for directory traversal and `.gitignore` handling.
- [`xmlwriter`](https://crates.io/crates/xmlwriter) for generating XML output.
