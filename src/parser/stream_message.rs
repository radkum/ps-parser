use std::fmt::Display;

// PowerShell-like output streams
#[derive(Debug, Clone, PartialEq)]
pub enum PowerShellStream {
    Success, // Stream 1 - regular output
    Error,   // Stream 2 - errors
    Warning, // Stream 3 - warnings
    Verbose, // Stream 4 - verbose messages
}

impl Display for PowerShellStream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            PowerShellStream::Success => "",
            PowerShellStream::Error => "ERROR",
            PowerShellStream::Warning => "WARNING",
            PowerShellStream::Verbose => "VERBOSE",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Clone)]
pub struct StreamMessage {
    pub content: String,
    pub stream: PowerShellStream,
    pub timestamp: std::time::SystemTime,
}

impl Display for StreamMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.stream == PowerShellStream::Success {
            write!(f, "{}", self.content)
        } else {
            write!(
                f,
                "[{}] {}: {}",
                self.stream,
                self.timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                self.content
            )
        }
    }
}

impl StreamMessage {
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn success(content: String) -> Self {
        StreamMessage {
            content,
            stream: PowerShellStream::Success,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn warning(message: String) -> Self {
        StreamMessage {
            content: format!("WARNING: {}", message),
            stream: PowerShellStream::Warning,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn error(message: String) -> Self {
        StreamMessage {
            content: format!("ERROR: {}", message),
            stream: PowerShellStream::Error,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn verbose(message: String) -> Self {
        StreamMessage {
            content: format!("VERBOSE: {}", message),
            stream: PowerShellStream::Verbose,
            timestamp: std::time::SystemTime::now(),
        }
    }
}
