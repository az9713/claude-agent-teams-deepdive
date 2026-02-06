use std::io::IsTerminal;

use indicatif::{ProgressBar, ProgressStyle};

pub struct ScanProgress {
    bar: Option<ProgressBar>,
}

impl ScanProgress {
    pub fn new(total: u64) -> Self {
        let bar = if std::io::stderr().is_terminal() && total > 1000 {
            let pb = ProgressBar::new(total);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
                    .unwrap()
                    .progress_chars("=>-"),
            );
            Some(pb)
        } else {
            None
        };
        ScanProgress { bar }
    }

    pub fn inc(&self) {
        if let Some(ref bar) = self.bar {
            bar.inc(1);
        }
    }

    pub fn finish(&self) {
        if let Some(ref bar) = self.bar {
            bar.finish_and_clear();
        }
    }
}
