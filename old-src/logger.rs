use core::time::Duration;
use std::fmt::Display;
use colored::Colorize;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};

pub struct Logger {
    bar: ProgressBar,
    running: bool,
}

impl Logger {
    pub fn new() -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(ProgressStyle::with_template("🕊️  {msg} 🕊️ ").unwrap());
        bar.set_message("building the blog");
        bar.enable_steady_tick(Duration::from_millis(100));

        Self {
            bar,
            running: true,
        }
    }

    fn stop(&mut self) {
        if self.running {
            self.running = false;
            self.bar.set_message("building complete");
            self.bar.tick();
            self.bar.finish();
        }
    }
    
    pub fn abort(&mut self) {
        if self.running {
            self.running = false;
            self.bar.set_message("building failed");
            self.bar.tick();
            self.bar.finish();
        }
    }

    fn emit<L: Display, S: AsRef<str>>(&self, level: L, msg: S) {
        self.bar.println(format!("{} {}", level, msg.as_ref()));
    }

    pub fn info<S: AsRef<str>>(&self, msg: S) {
        self.emit("[INFO]".green().bold(), msg);
    }

    pub fn debug<S: AsRef<str>>(&self, _msg: S) {
        #[cfg(debug_assertions)]
        {
            self.emit("[DEBUG]".black().on_white(), _msg);
        }
    }

    pub fn error<S: AsRef<str>>(&self, msg: S) {
        self.emit("[ERROR]".red().bold(), msg);
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style() {
        let logger = Logger::new();
        logger.info("info");
        logger.debug("debug");
        logger.error("error");

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
