use std::io::Write;

/// Writes formatted frame data to an output destination.
///
/// Flushing is an internal implementation detail of each writer.
pub trait Writer {
    fn write(&mut self, buf: &[u8]) -> eyre::Result<()>;
}

/// Writes formatted output to stdout, flushing after every write for live log viewing.
pub struct StdoutWriter {
    inner: std::io::BufWriter<std::io::Stdout>,
}

impl Default for StdoutWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl StdoutWriter {
    pub fn new() -> Self {
        Self {
            inner: std::io::BufWriter::new(std::io::stdout()),
        }
    }
}

impl Writer for StdoutWriter {
    fn write(&mut self, buf: &[u8]) -> eyre::Result<()> {
        self.inner.write_all(buf)?;
        self.inner.flush()?;
        Ok(())
    }
}
