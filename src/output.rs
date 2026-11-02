//! Output sinks the progress bar can render to.

use core::fmt;

/// Something a [`ProgressBar`](crate::pb::ProgressBar) can write to.
///
/// Any [`fmt::Write`] implementor can be used (a UART, an RTT channel, a `String`...):
/// implement this trait for it, overriding [`Output::flush`] if your sink is buffered.
///
/// ```
/// use core::fmt;
/// use progress_bar::Output;
///
/// struct Uart;
///
/// impl fmt::Write for Uart {
///     fn write_str(&mut self, s: &str) -> fmt::Result {
///         // send `s` over the wire
///         Ok(())
///     }
/// }
///
/// impl Output for Uart {}
/// ```
///
/// If your sink implements [`embedded_io::Write`](https://docs.rs/embedded-io) instead,
/// wrap it in `EmbeddedIo` (requires the `embedded-io` feature).
pub trait Output: fmt::Write {
    /// Flushes buffered output, called after each full redraw of the bar.
    fn flush(&mut self) {}
}

#[cfg(feature = "std")]
impl Output for std::string::String {}

/// Writes to the process' standard output. This is the default output.
///
/// It goes through `print!` rather than `io::stdout()`, so that `cargo test` captures the output.
#[cfg(feature = "std")]
#[derive(Debug, Default, Clone, Copy)]
pub struct Stdout;

#[cfg(feature = "std")]
impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        std::print!("{s}");
        Ok(())
    }

    // A single `print!` takes the stdout lock once, so a line can't be interleaved with other threads' output.
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        std::print!("{args}");
        Ok(())
    }
}

#[cfg(feature = "std")]
impl Output for Stdout {
    fn flush(&mut self) {
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
}

/// Adapter rendering to any [`embedded_io::Write`] implementor (UARTs from most embedded HALs).
///
/// ```
/// use progress_bar::{pb::ProgressBar, EmbeddedIo, NoClock};
///
/// let mut buf = [0u8; 256];
/// let mut progress_bar = ProgressBar::<_, _, &str>::with_output(10, EmbeddedIo(&mut buf[..]), NoClock);
/// progress_bar.inc();
/// ```
#[cfg(feature = "embedded-io")]
#[derive(Debug, Default, Clone, Copy)]
pub struct EmbeddedIo<T>(pub T);

#[cfg(feature = "embedded-io")]
impl<T> EmbeddedIo<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[cfg(feature = "embedded-io")]
impl<T: embedded_io::Write> fmt::Write for EmbeddedIo<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

#[cfg(feature = "embedded-io")]
impl<T: embedded_io::Write> Output for EmbeddedIo<T> {
    fn flush(&mut self) {
        let _ = self.0.flush();
    }
}

#[cfg(all(test, feature = "embedded-io"))]
mod tests {
    use super::*;
    use crate::{pb::ProgressBar, NoClock};

    #[test]
    fn embedded_io_output() {
        let mut buf = [0u8; 256];
        let mut progress_bar = ProgressBar::<_, _, &str>::with_output(4, EmbeddedIo(&mut buf[..]), NoClock);
        progress_bar.set_width(4);
        progress_bar.inc();
        let remaining = progress_bar.into_output().into_inner().len();
        let written = core::str::from_utf8(&buf[..256 - remaining]).unwrap();
        assert!(written.ends_with(" [>   ] 1/4\n\x1B[1A"));
    }
}
