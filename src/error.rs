use std::fmt;
use std::path::PathBuf;
use owo_colors::OwoColorize;

/// Parser error with source location information for beautiful diagnostics
#[derive(Debug)]
pub struct ParserError {
    pub(crate) kind: ErrorKind,
    pub(crate) file: Option<PathBuf>,
    pub(crate) span: Option<(usize, usize)>,
    pub(crate) source_content: Option<String>,
}

#[derive(Debug)]
pub enum ErrorKind {
    /// File could not be found or opened
    FileNotFound { path: PathBuf, io_error: String },
    
    /// File contains invalid UTF-8
    InvalidUtf8 { path: PathBuf },
    
    /// Front matter YAML is invalid
    InvalidFrontMatter { message: String },
    
    /// Required field is missing from front matter
    MissingRequiredField { field: String },
    
    /// Circular dependency detected in page graph
    CircularDependency { chain: Vec<String> },
    
    /// Referenced page does not exist
    PageNotFound { id: String, context: String },
    
    /// Path is not under the expected base directory
    PathNotUnderBase { path: PathBuf, base: PathBuf },
    
    /// Path has no parent directory
    NoParentDir { path: PathBuf },
    
    /// Path has no file stem
    NoFileStem { path: PathBuf },
    
    /// Invalid child reference in front matter
    InvalidChildReference {
        parent: PathBuf,
        reference: String,
        reason: String,
    },
    
    /// Referenced child file not found
    MissingChildReference {
        parent_id: String,
        reference: String,
    },
    
    /// Generic error with message
    Other { message: String },
}

impl ParserError {
    /// Create a simple error with just a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Other { message: message.into() },
            file: None,
            span: None,
            source_content: None,
        }
    }
    
    /// Create an error for a file that wasn't found
    pub fn file_not_found(path: PathBuf, io_error: impl fmt::Display) -> Self {
        Self {
            kind: ErrorKind::FileNotFound { 
                path: path.clone(), 
                io_error: io_error.to_string() 
            },
            file: Some(path),
            span: None,
            source_content: None,
        }
    }
    
    /// Create an error for invalid UTF-8 in a file
    pub fn invalid_utf8(path: PathBuf) -> Self {
        Self {
            kind: ErrorKind::InvalidUtf8 { path: path.clone() },
            file: Some(path),
            span: None,
            source_content: None,
        }
    }
    
    /// Create an error for invalid front matter with source context
    pub fn invalid_front_matter(
        path: PathBuf,
        message: String,
        source: String,
        span: (usize, usize),
    ) -> Self {
        Self {
            kind: ErrorKind::InvalidFrontMatter { message },
            file: Some(path),
            span: Some(span),
            source_content: Some(source),
        }
    }
    
    /// Create an error for a path not under base directory
    pub fn path_not_under_base(path: PathBuf, base: PathBuf) -> Self {
        Self {
            kind: ErrorKind::PathNotUnderBase { path, base },
            file: None,
            span: None,
            source_content: None,
        }
    }
    
    pub fn invalid_child_reference(
        parent: PathBuf,
        reference: String,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            kind: ErrorKind::InvalidChildReference {
                parent,
                reference,
                reason: reason.into(),
            },
            file: None,
            span: None,
            source_content: None,
        }
    }
    
    pub fn missing_child_reference(
        parent_id: impl std::fmt::Debug,
        reference: String,
    ) -> Self {
        Self {
            kind: ErrorKind::MissingChildReference {
                parent_id: format!("{:?}", parent_id),
                reference,
            },
            file: None,
            span: None,
            source_content: None,
        }
    }
    
    /// Print this error with colored output and context
    pub fn print(&self) {
        // Use owo-colors for cross-platform color support with NO_COLOR handling
        if let Some(ref file) = self.file {
            eprintln!("{} in {}:", "Error".red().bold(), file.display().bold());
        } else {
            eprintln!("{}:", "Error".red().bold());
        }
        
        eprintln!("  {}", self);
        
        // Show source context if available
        if let Some(ref content) = self.source_content {
            if let Some(span) = self.span {
                eprintln!("\n  {}:", "Source".dimmed());
                let lines: Vec<&str> = content.lines().collect();
                let start_line = content[..span.0].matches('\n').count();
                let end_line = content[..span.1.min(content.len())].matches('\n').count();
                
                for (i, line) in lines.iter().enumerate().skip(start_line.saturating_sub(1)).take(end_line - start_line + 3) {
                    if i == start_line {
                        eprintln!("  {} {}", format!("{:>4} |", i + 1).red().bold(), line);
                    } else {
                        eprintln!("  {} {}", format!("{:>4} |", i + 1).dimmed(), line);
                    }
                }
            }
        }
        
        // Show help if available
        match &self.kind {
            ErrorKind::InvalidFrontMatter { .. } => {
                eprintln!("\n{} Ensure your front matter is valid YAML between --- markers", "Help:".cyan().bold());
            }
            ErrorKind::MissingRequiredField { field } => {
                eprintln!("\n{} Add '{}: <value>' to your front matter", "Help:".cyan().bold(), field);
            }
            _ => {}
        }
        
        eprintln!();
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::FileNotFound { path, io_error } => {
                write!(f, "File not found: {} ({})", path.display(), io_error)
            }
            ErrorKind::InvalidUtf8 { path } => {
                write!(f, "File contains invalid UTF-8: {}", path.display())
            }
            ErrorKind::InvalidFrontMatter { message } => {
                write!(f, "Invalid front matter: {}", message)
            }
            ErrorKind::MissingRequiredField { field } => {
                write!(f, "Missing required field: {}", field)
            }
            ErrorKind::CircularDependency { chain } => {
                write!(f, "Circular dependency detected: {}", chain.join(" -> "))
            }
            ErrorKind::PageNotFound { id, context } => {
                write!(f, "Page not found: {} ({})", id, context)
            }
            ErrorKind::PathNotUnderBase { path, base } => {
                write!(f, "Path {:?} is not under base directory {:?}", path, base)
            }
            ErrorKind::NoParentDir { path } => {
                write!(f, "Path has no parent directory: {:?}", path)
            }
            ErrorKind::NoFileStem { path } => {
                write!(f, "Path has no file stem: {}", path.display())
            }
            ErrorKind::InvalidChildReference { parent, reference, reason } => {
                write!(f, "Invalid child reference '{}' in {}: {}", reference, parent.display(), reason)
            }
            ErrorKind::MissingChildReference { parent_id, reference } => {
                write!(f, "Missing child reference '{}' in page {}", reference, parent_id)
            }
            ErrorKind::Other { message } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for ParserError {}

/// Convert from anyhow::Error for compatibility
impl From<anyhow::Error> for ParserError {
    fn from(err: anyhow::Error) -> Self {
        ParserError::new(err.to_string())
    }
}

/// Result type using ParserError
pub type Result<T> = std::result::Result<T, ParserError>;

/// Collects multiple errors for batch reporting
#[derive(Default)]
pub struct ErrorCollector {
    errors: Vec<ParserError>,
    warnings: Vec<ParserError>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add an error to the collection
    pub fn add_error(&mut self, error: ParserError) {
        self.errors.push(error);
    }
    
    /// Add a warning to the collection
    pub fn add_warning(&mut self, warning: ParserError) {
        self.warnings.push(warning);
    }
    
    /// Check if any errors were collected
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
    
    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
    
    /// Print all collected errors and warnings
    pub fn print_all(&self) {
        use owo_colors::OwoColorize;
        if self.errors.is_empty() && self.warnings.is_empty() {
            return;
        }
        
        let separator = "━".repeat(60);
        eprintln!("\n{}", separator.dimmed());
        
        let error_text = format!("{} error(s)", self.errors.len());
        let warning_text = format!("{} warning(s)", self.warnings.len());
        
        eprintln!(
            "Found {} and {}:",
            if self.errors.len() > 0 { error_text.red().bold().to_string() } else { error_text },
            if self.warnings.len() > 0 { warning_text.yellow().to_string() } else { warning_text }
        );
        eprintln!("{}\n", separator.dimmed());
        
        // Print warnings first
        for warning in &self.warnings {
            eprintln!("{}", "Warning:".yellow().bold());
            eprintln!("  {}\n", warning);
        }
        
        // Print errors
        for error in &self.errors {
            error.print();
        }
        
        eprintln!("{}", separator.dimmed());
        
        let error_summary = format!("{} error(s)", self.errors.len());
        let warning_summary = format!("{} warning(s)", self.warnings.len());
        
        eprintln!(
            "{} {} {}",
            "Summary:".bold(),
            if self.errors.len() > 0 { error_summary.red().to_string() } else { error_summary },
            if self.warnings.len() > 0 { warning_summary.yellow().to_string() } else { warning_summary }
        );
        eprintln!("{}", separator.dimmed());
    }
    
    /// Convert to Result, failing if errors exist
    pub fn into_result(self) -> std::result::Result<(), String> {
        if self.has_errors() {
            Err(format!("{} error(s) occurred", self.errors.len()))
        } else {
            Ok(())
        }
    }
}
