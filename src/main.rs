use clap::{Parser, ValueEnum};
// Remove glob import
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
// Remove walkdir import if no longer needed elsewhere
// use walkdir::WalkDir;
use xmlwriter::{Options, XmlWriter};

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum OutputFormat {
    Text,
    Xml,
}

#[derive(Parser, Debug)]
#[command(
    author = "Prithvish Baidya (d4mr)",
    version = "0.1.0",
    about = "Generates directory structure and file contents for LLM context.",
    long_about = "Processes a directory, creating a structure view (text tree or XML) \
                 and concatenating readable file contents. Uses gitignore-style patterns \
                 for exclusion and optionally respects .gitignore files. Defaults to the \
                 current directory if none is specified."
)]
struct Args {
    /// The directory to process.
    #[arg(short = 'd', long, value_name = "DIRECTORY", default_value = ".")]
    directory: PathBuf,

    /// Output format.
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Maximum depth to traverse (0 means no limit).
    #[arg(
        short = 'm',
        long = "max-depth",
        value_name = "DEPTH",
        default_value_t = 0
    )]
    max_depth: usize,

    /// Gitignore-style patterns to exclude files/directories. Applied *after* gitignore rules.
    #[arg(short = 'x', long, value_name = "PATTERN", num_args = 0..)]
    // <-- Changed value_name hint
    exclude: Vec<String>,

    /// Respect .gitignore files found in the directory structure.
    #[arg(long, default_value_t = true)]
    respect_gitignore: bool,
}

// --- Data Structures (Unchanged) ---
struct FileInfo {
    path: PathBuf,
    content: String,
}

struct DirEntryInfo {
    // path: PathBuf, // Path to the directory entry
    name: String,
    is_dir: bool,
    depth: usize,
}
// ---------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Canonicalize the path (handles '.' correctly)
    let root_path = match args.directory.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if args.directory == PathBuf::from(".") && e.kind() == io::ErrorKind::NotFound {
                eprintln!("Error: Current directory '.' not found or inaccessible.");
            } else {
                eprintln!(
                    "Error: Could not access directory '{}': {}",
                    args.directory.display(),
                    e
                );
            }
            std::process::exit(1);
        }
    };

    if !root_path.is_dir() {
        eprintln!("Error: '{}' is not a valid directory.", root_path.display());
        std::process::exit(1);
    }

    // --- Build Overrides for --exclude patterns using 'ignore' crate ---
    let mut override_builder = OverrideBuilder::new(&root_path);
    for pattern_str in &args.exclude {
        // Prepend '!' to make it an ignore pattern for the 'ignore' crate's OverrideBuilder
        // The pattern itself should follow gitignore syntax.
        let ignore_pattern = format!("!{}", pattern_str);
        if let Err(e) = override_builder.add(&ignore_pattern) {
            eprintln!(
                "Warning: Invalid exclude pattern '{}': {} (Ignoring)",
                pattern_str,
                e // Use original pattern_str in warning
            );
        }
    }
    // Build the override rules. This can fail if patterns are fundamentally broken.
    let overrides = match override_builder.build() {
        Ok(ov) => ov,
        Err(e) => {
            eprintln!("Error: Failed to build exclusion rules: {}", e);
            std::process::exit(1);
        }
    };

    // --- Collect directory structure and file contents using 'ignore' crate ---
    let mut dir_entries: Vec<DirEntryInfo> = Vec::new();
    let mut file_contents: Vec<FileInfo> = Vec::new();

    eprintln!("Processing directory: {}", root_path.display());
    if args.respect_gitignore {
        eprintln!("Respecting .gitignore files.");
    }
    if !args.exclude.is_empty() {
        eprintln!("Applying exclude patterns: {:?}", args.exclude);
    }

    // --- Configure WalkBuilder ---
    let mut walk_builder = WalkBuilder::new(&root_path);
    walk_builder
        .git_ignore(args.respect_gitignore) // Control .gitignore handling
        .ignore(false) // Don't use .ignore files
        .git_global(false) // Don't use global gitignore
        .git_exclude(false) // Don't use .git/info/exclude
        .overrides(overrides); // Apply command-line --exclude patterns

    if args.max_depth > 0 {
        // Add 1 because WalkBuilder depth includes the root (depth 0)
        walk_builder.max_depth(Some(args.max_depth + 1));
    }

    // --- Iterate ---
    for entry_result in walk_builder.build() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Warning: Error accessing entry: {}", e);
                continue;
            }
        };

        // Skip the root directory itself (depth 0)
        if entry.depth() == 0 {
            continue;
        }

        let path = entry.path().to_path_buf();
        // Adjust depth to be relative to the *start* directory
        let depth = entry.depth().saturating_sub(1);
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map_or(false, |ft| ft.is_dir());

        dir_entries.push(DirEntryInfo {
            // path: path.clone(),
            name,
            is_dir,
            depth,
        });

        if !is_dir {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    file_contents.push(FileInfo { path, content });
                }
                Err(e) => {
                    if e.kind() != io::ErrorKind::InvalidData {
                        writeln!(
                            io::stderr(),
                            "Warning: Could not read file '{}': {} (Skipping content)",
                            path.display(),
                            e
                        )?;
                    } else {
                        writeln!(
                            io::stderr(),
                            "Info: Skipping binary or non-UTF8 file: '{}'",
                            path.display()
                        )?;
                    }
                }
            }
        }
    }

    // --- Generate Output (Unchanged) ---
    match args.format {
        OutputFormat::Text => {
            generate_text_output(&root_path, &dir_entries, &file_contents);
        }
        OutputFormat::Xml => {
            generate_xml_output(&root_path, &dir_entries, &file_contents);
        }
    }

    Ok(())
}

// --- Text Output Generation ---
fn generate_text_output(
    root_path: &Path,
    dir_entries: &[DirEntryInfo],
    file_contents: &[FileInfo],
) {
    println!("--- Directory Tree ---");
    println!(
        "{}/",
        root_path.file_name().unwrap_or_default().to_string_lossy()
    );
    for entry in dir_entries {
        let indent = "  ".repeat(entry.depth + 1);
        let suffix = if entry.is_dir { "/" } else { "" };
        println!("{}{}{}", indent, entry.name, suffix);
    }

    println!("\n--- File Contents ---");
    if file_contents.is_empty() {
        println!("(No readable files found or all were excluded/ignored)");
    } else {
        for file_info in file_contents {
            println!("========================================");
            let display_path = file_info
                .path
                .strip_prefix(root_path)
                .unwrap_or(&file_info.path);
            println!("File: {}", display_path.display());
            println!("========================================");
            println!("{}", file_info.content.trim());
            println!();
        }
    }
}

// --- XML Output Generation ---
fn generate_xml_output(root_path: &Path, dir_entries: &[DirEntryInfo], file_contents: &[FileInfo]) {
    let mut xw = XmlWriter::new(Options::default());
    xw.start_element("projectContext");
    xw.write_attribute("rootPath", &root_path.to_string_lossy());

    xw.start_element("tree");
    xw.start_element("dir");
    xw.write_attribute(
        "name",
        &root_path.file_name().unwrap_or_default().to_string_lossy(),
    );
    let mut stack: Vec<usize> = vec![0];

    for entry in dir_entries {
        let entry_xml_depth = entry.depth + 1;

        while *stack.last().unwrap_or(&0) >= entry_xml_depth {
            xw.end_element();
            stack.pop();
        }

        if entry.is_dir {
            xw.start_element("dir");
            xw.write_attribute("name", &entry.name);
            stack.push(entry_xml_depth);
        } else {
            xw.start_element("file");
            xw.write_attribute("name", &entry.name);
            xw.end_element();
        }
    }
    while stack.len() > 1 {
        xw.end_element();
        stack.pop();
    }

    xw.end_element(); // Close root dir
    xw.end_element(); // Close <tree>

    xw.start_element("fileContents");
    if file_contents.is_empty() {
        xw.write_comment(" No readable files found or all were excluded/ignored ");
    } else {
        for file_info in file_contents {
            let display_path = file_info
                .path
                .strip_prefix(root_path)
                .unwrap_or(&file_info.path);
            xw.start_element("file");
            xw.write_attribute("path", &display_path.to_string_lossy());
            xw.write_text(&file_info.content);
            xw.end_element(); // </file>
        }
    }
    xw.end_element(); // </fileContents>
    xw.end_element(); // </projectContext>
    print!("{}", xw.end_document());
}
