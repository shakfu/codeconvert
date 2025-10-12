use clap::{Parser, Subcommand};
use codeconvert_core::{
    CaseConverter, CaseFormat, CaseTransform, EmojiOptions, EmojiTransformer, FileRenamer,
    RenameOptions, SpaceReplace, WhitespaceCleaner, WhitespaceOptions,
};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
use logging_timer::time;
use simplelog::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "codeconvert",
    version = "0.2.0",
    about = "Code transformation tool for case conversion and cleaning",
    long_about = "A modular code transformation framework.\n\n\
                  Commands:\n\
                  - convert: Convert between case formats\n\
                  - clean: Remove trailing whitespace\n\
                  - emojis: Remove or replace emojis with text alternatives\n\
                  - rename_files: Rename files with various transformations"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output (can be used multiple times: -v, -vv, -vvv)
    #[arg(short = 'v', long = "verbose", global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all output except errors
    #[arg(short = 'q', long = "quiet", global = true)]
    quiet: bool,

    /// Write logs to file
    #[arg(long = "log-file", global = true)]
    log_file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert between case formats
    #[command(group(clap::ArgGroup::new("from").required(true).multiple(false)))]
    #[command(group(clap::ArgGroup::new("to").required(true).multiple(false)))]
    Convert {
        /// Convert FROM camelCase
        #[arg(long = "from-camel", group = "from")]
        from_camel: bool,

        /// Convert FROM PascalCase
        #[arg(long = "from-pascal", group = "from")]
        from_pascal: bool,

        /// Convert FROM snake_case
        #[arg(long = "from-snake", group = "from")]
        from_snake: bool,

        /// Convert FROM SCREAMING_SNAKE_CASE
        #[arg(long = "from-screaming-snake", group = "from")]
        from_screaming_snake: bool,

        /// Convert FROM kebab-case
        #[arg(long = "from-kebab", group = "from")]
        from_kebab: bool,

        /// Convert FROM SCREAMING-KEBAB-CASE
        #[arg(long = "from-screaming-kebab", group = "from")]
        from_screaming_kebab: bool,

        /// Convert TO camelCase
        #[arg(long = "to-camel", group = "to")]
        to_camel: bool,

        /// Convert TO PascalCase
        #[arg(long = "to-pascal", group = "to")]
        to_pascal: bool,

        /// Convert TO snake_case
        #[arg(long = "to-snake", group = "to")]
        to_snake: bool,

        /// Convert TO SCREAMING_SNAKE_CASE
        #[arg(long = "to-screaming-snake", group = "to")]
        to_screaming_snake: bool,

        /// Convert TO kebab-case
        #[arg(long = "to-kebab", group = "to")]
        to_kebab: bool,

        /// Convert TO SCREAMING-KEBAB-CASE
        #[arg(long = "to-screaming-kebab", group = "to")]
        to_screaming_kebab: bool,

        /// The directory or file to convert
        path: PathBuf,

        /// Convert files recursively
        #[arg(short = 'r', long)]
        recursive: bool,

        /// Dry run the conversion
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,

        /// Prefix to add to all converted words
        #[arg(long, default_value = "")]
        prefix: String,

        /// Suffix to add to all converted words
        #[arg(long, default_value = "")]
        suffix: String,

        /// Strip prefix before conversion (e.g., 'm_' from 'm_userName')
        #[arg(long = "strip-prefix")]
        strip_prefix: Option<String>,

        /// Strip suffix before conversion
        #[arg(long = "strip-suffix")]
        strip_suffix: Option<String>,

        /// Replace prefix (from) before conversion (e.g., 'I' in 'IUserService')
        #[arg(long = "replace-prefix-from")]
        replace_prefix_from: Option<String>,

        /// Replace prefix (to) before conversion (e.g., 'Abstract')
        #[arg(long = "replace-prefix-to", requires = "replace_prefix_from")]
        replace_prefix_to: Option<String>,

        /// Replace suffix (from) before conversion
        #[arg(long = "replace-suffix-from")]
        replace_suffix_from: Option<String>,

        /// Replace suffix (to) before conversion
        #[arg(long = "replace-suffix-to", requires = "replace_suffix_from")]
        replace_suffix_to: Option<String>,

        /// Glob pattern to filter files
        #[arg(long)]
        glob: Option<String>,

        /// Regex pattern to filter which words get converted
        #[arg(long = "word-filter")]
        word_filter: Option<String>,
    },

    /// Remove trailing whitespace from files
    Clean {
        /// The directory or file to clean
        path: PathBuf,

        /// Process files recursively
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't modify files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,
    },

    /// Remove or replace emojis with text alternatives
    Emojis {
        /// The directory or file to process
        path: PathBuf,

        /// Process files recursively [default: true]
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't modify files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process (default: .md, .txt, and common source files)
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,

        /// Replace task completion emojis with text (e.g., ✅ -> [x]) [default: true]
        #[arg(long = "replace-task", default_value_t = true)]
        replace_task: bool,

        /// Remove all other emojis [default: true]
        #[arg(long = "remove-other", default_value_t = true)]
        remove_other: bool,
    },

    /// Rename files with various transformations
    #[command(name = "rename_files")]
    RenameFiles {
        /// The directory or file to rename
        path: PathBuf,

        /// Process directories recursively [default: true]
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't rename files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// Convert to lowercase
        #[arg(long = "to-lowercase")]
        to_lowercase: bool,

        /// Convert to UPPERCASE
        #[arg(long = "to-uppercase")]
        to_uppercase: bool,

        /// Capitalize (first letter uppercase, rest lowercase)
        #[arg(long = "to-capitalize")]
        to_capitalize: bool,

        /// Replace separators (spaces, hyphens, underscores) with underscores
        #[arg(long = "underscored")]
        underscored: bool,

        /// Replace separators (spaces, hyphens, underscores) with hyphens
        #[arg(long = "hyphenated")]
        hyphenated: bool,

        /// Add prefix to filename
        #[arg(long = "add-prefix")]
        add_prefix: Option<String>,

        /// Remove prefix from filename
        #[arg(long = "rm-prefix")]
        rm_prefix: Option<String>,

        /// Add suffix to filename (before extension)
        #[arg(long = "add-suffix")]
        add_suffix: Option<String>,

        /// Remove suffix from filename (before extension)
        #[arg(long = "rm-suffix")]
        rm_suffix: Option<String>,
    },
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: u8, quiet: bool, log_file: Option<PathBuf>) -> anyhow::Result<()> {
    let log_level = if quiet {
        LevelFilter::Error
    } else {
        match verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    };

    let config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_thread_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        .build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        log_level,
        config.clone(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];

    if let Some(log_path) = log_file {
        let file = std::fs::File::create(&log_path)?;
        loggers.push(WriteLogger::new(LevelFilter::Debug, config, file));
        eprintln!("Logging to file: {}", log_path.display());
    }

    CombinedLogger::init(loggers)?;

    debug!("Logging initialized with level: {:?}", log_level);
    Ok(())
}

/// Create a progress spinner
fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    spinner
}

fn determine_case_format(
    from_camel: bool,
    from_pascal: bool,
    from_snake: bool,
    from_screaming_snake: bool,
    from_kebab: bool,
    _from_screaming_kebab: bool,
) -> CaseFormat {
    if from_camel {
        CaseFormat::CamelCase
    } else if from_pascal {
        CaseFormat::PascalCase
    } else if from_snake {
        CaseFormat::SnakeCase
    } else if from_screaming_snake {
        CaseFormat::ScreamingSnakeCase
    } else if from_kebab {
        CaseFormat::KebabCase
    } else {
        CaseFormat::ScreamingKebabCase
    }
}

#[time("info")]
fn run_convert(
    from_camel: bool,
    from_pascal: bool,
    from_snake: bool,
    from_screaming_snake: bool,
    from_kebab: bool,
    from_screaming_kebab: bool,
    to_camel: bool,
    to_pascal: bool,
    to_snake: bool,
    to_screaming_snake: bool,
    to_kebab: bool,
    to_screaming_kebab: bool,
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
    prefix: String,
    suffix: String,
    strip_prefix: Option<String>,
    strip_suffix: Option<String>,
    replace_prefix_from: Option<String>,
    replace_prefix_to: Option<String>,
    replace_suffix_from: Option<String>,
    replace_suffix_to: Option<String>,
    glob: Option<String>,
    word_filter: Option<String>,
) -> anyhow::Result<()> {
    let from_format = determine_case_format(
        from_camel,
        from_pascal,
        from_snake,
        from_screaming_snake,
        from_kebab,
        from_screaming_kebab,
    );

    let to_format = determine_case_format(
        to_camel,
        to_pascal,
        to_snake,
        to_screaming_snake,
        to_kebab,
        to_screaming_kebab,
    );

    info!(
        "Converting from {:?} to {:?}",
        from_format, to_format
    );
    info!("Target path: {}", path.display());
    info!("Recursive: {}, Dry run: {}", recursive, dry_run);

    if let Some(ref exts) = extensions {
        debug!("File extensions: {:?}", exts);
    }
    if !prefix.is_empty() {
        debug!("Prefix: '{}'", prefix);
    }
    if !suffix.is_empty() {
        debug!("Suffix: '{}'", suffix);
    }
    if let Some(ref pattern) = glob {
        debug!("Glob pattern: '{}'", pattern);
    }
    if let Some(ref filter) = word_filter {
        debug!("Word filter: '{}'", filter);
    }

    let spinner = create_spinner("Processing files...");

    let converter = CaseConverter::new(
        from_format,
        to_format,
        extensions,
        recursive,
        dry_run,
        prefix,
        suffix,
        strip_prefix,
        strip_suffix,
        replace_prefix_from,
        replace_prefix_to,
        replace_suffix_from,
        replace_suffix_to,
        glob,
        word_filter,
    )?;

    let result = converter.process_directory(&path);

    spinner.finish_and_clear();

    match result {
        Ok(_) => {
            info!("Conversion completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Conversion failed: {}", e);
            Err(e)
        }
    }
}

#[time("info")]
fn run_clean(
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
) -> anyhow::Result<()> {
    info!("Cleaning whitespace from: {}", path.display());
    info!("Recursive: {}, Dry run: {}", recursive, dry_run);

    if let Some(ref exts) = extensions {
        debug!("File extensions: {:?}", exts);
    }

    let mut options = WhitespaceOptions::default();
    options.recursive = recursive;
    options.dry_run = dry_run;

    if let Some(exts) = extensions {
        options.file_extensions = exts;
    }

    let spinner = create_spinner("Cleaning files...");

    let cleaner = WhitespaceCleaner::new(options);
    let (files, lines) = cleaner.process(&path)?;

    spinner.finish_and_clear();

    if files > 0 {
        let prefix = if dry_run { "[DRY-RUN] " } else { "" };
        info!(
            "{}Cleaned {} lines in {} file(s)",
            prefix, lines, files
        );
        println!(
            "{}Cleaned {} lines in {} file(s)",
            prefix, lines, files
        );
    } else {
        info!("No files needed cleaning");
        println!("No files needed cleaning");
    }

    Ok(())
}

#[time("info")]
fn run_emojis(
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
    replace_task: bool,
    remove_other: bool,
) -> anyhow::Result<()> {
    info!("Processing emojis from: {}", path.display());
    info!("Recursive: {}, Dry run: {}", recursive, dry_run);
    info!(
        "Replace task emojis: {}, Remove other emojis: {}",
        replace_task, remove_other
    );

    if let Some(ref exts) = extensions {
        debug!("File extensions: {:?}", exts);
    }

    let mut options = EmojiOptions::default();
    options.recursive = recursive;
    options.dry_run = dry_run;
    options.replace_task_emojis = replace_task;
    options.remove_other_emojis = remove_other;

    if let Some(exts) = extensions {
        options.file_extensions = exts;
    }

    let spinner = create_spinner("Transforming emojis...");

    let transformer = EmojiTransformer::new(options);
    let (files, changes) = transformer.process(&path)?;

    spinner.finish_and_clear();

    if files > 0 {
        let prefix = if dry_run { "[DRY-RUN] " } else { "" };
        info!(
            "{}Transformed emojis in {} file(s) ({} changes)",
            prefix, files, changes
        );
        println!(
            "{}Transformed emojis in {} file(s) ({} changes)",
            prefix, files, changes
        );
    } else {
        info!("No files contained emojis to transform");
        println!("No files contained emojis to transform");
    }

    Ok(())
}

#[time("info")]
fn run_rename(
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    to_lowercase: bool,
    to_uppercase: bool,
    to_capitalize: bool,
    underscored: bool,
    hyphenated: bool,
    add_prefix: Option<String>,
    rm_prefix: Option<String>,
    add_suffix: Option<String>,
    rm_suffix: Option<String>,
) -> anyhow::Result<()> {
    info!("Renaming files in: {}", path.display());
    info!("Recursive: {}, Dry run: {}", recursive, dry_run);

    let mut options = RenameOptions::default();
    options.recursive = recursive;
    options.dry_run = dry_run;

    // Set case transform (only one should be selected)
    if to_lowercase {
        options.case_transform = CaseTransform::Lowercase;
        debug!("Case transform: Lowercase");
    } else if to_uppercase {
        options.case_transform = CaseTransform::Uppercase;
        debug!("Case transform: Uppercase");
    } else if to_capitalize {
        options.case_transform = CaseTransform::Capitalize;
        debug!("Case transform: Capitalize");
    }

    // Set separator replacement (only one should be selected)
    if underscored {
        options.space_replace = SpaceReplace::Underscore;
        debug!("Separator replacement: Underscore");
    } else if hyphenated {
        options.space_replace = SpaceReplace::Hyphen;
        debug!("Separator replacement: Hyphen");
    }

    // Set prefix/suffix options
    options.add_prefix = add_prefix.clone();
    options.remove_prefix = rm_prefix.clone();
    options.add_suffix = add_suffix.clone();
    options.remove_suffix = rm_suffix.clone();

    if let Some(ref prefix) = add_prefix {
        debug!("Add prefix: '{}'", prefix);
    }
    if let Some(ref prefix) = rm_prefix {
        debug!("Remove prefix: '{}'", prefix);
    }
    if let Some(ref suffix) = add_suffix {
        debug!("Add suffix: '{}'", suffix);
    }
    if let Some(ref suffix) = rm_suffix {
        debug!("Remove suffix: '{}'", suffix);
    }

    let spinner = create_spinner("Renaming files...");

    let renamer = FileRenamer::new(options);
    let count = renamer.process(&path)?;

    spinner.finish_and_clear();

    if count > 0 {
        let prefix = if dry_run { "[DRY-RUN] " } else { "" };
        info!("{}Renamed {} file(s)", prefix, count);
        println!("{}Renamed {} file(s)", prefix, count);
    } else {
        info!("No files needed renaming");
        println!("No files needed renaming");
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if let Err(e) = init_logging(cli.verbose, cli.quiet, cli.log_file.clone()) {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }

    debug!("CLI arguments parsed successfully");

    let result = match cli.command {
        Commands::Convert {
            from_camel,
            from_pascal,
            from_snake,
            from_screaming_snake,
            from_kebab,
            from_screaming_kebab,
            to_camel,
            to_pascal,
            to_snake,
            to_screaming_snake,
            to_kebab,
            to_screaming_kebab,
            path,
            recursive,
            dry_run,
            extensions,
            prefix,
            suffix,
            strip_prefix,
            strip_suffix,
            replace_prefix_from,
            replace_prefix_to,
            replace_suffix_from,
            replace_suffix_to,
            glob,
            word_filter,
        } => {
            debug!("Running convert subcommand");
            run_convert(
                from_camel,
                from_pascal,
                from_snake,
                from_screaming_snake,
                from_kebab,
                from_screaming_kebab,
                to_camel,
                to_pascal,
                to_snake,
                to_screaming_snake,
                to_kebab,
                to_screaming_kebab,
                path,
                recursive,
                dry_run,
                extensions,
                prefix,
                suffix,
                strip_prefix,
                strip_suffix,
                replace_prefix_from,
                replace_prefix_to,
                replace_suffix_from,
                replace_suffix_to,
                glob,
                word_filter,
            )
        }

        Commands::Clean {
            path,
            recursive,
            dry_run,
            extensions,
        } => {
            debug!("Running clean subcommand");
            run_clean(path, recursive, dry_run, extensions)
        }

        Commands::Emojis {
            path,
            recursive,
            dry_run,
            extensions,
            replace_task,
            remove_other,
        } => {
            debug!("Running emojis subcommand");
            run_emojis(path, recursive, dry_run, extensions, replace_task, remove_other)
        }

        Commands::RenameFiles {
            path,
            recursive,
            dry_run,
            to_lowercase,
            to_uppercase,
            to_capitalize,
            underscored,
            hyphenated,
            add_prefix,
            rm_prefix,
            add_suffix,
            rm_suffix,
        } => {
            debug!("Running rename subcommand");
            run_rename(
                path,
                recursive,
                dry_run,
                to_lowercase,
                to_uppercase,
                to_capitalize,
                underscored,
                hyphenated,
                add_prefix,
                rm_prefix,
                add_suffix,
                rm_suffix,
            )
        }
    };

    if let Err(ref e) = result {
        error!("Operation failed: {}", e);
    } else {
        debug!("Operation completed successfully");
    }

    result
}
