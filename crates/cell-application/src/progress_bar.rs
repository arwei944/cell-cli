use std::io::Write;
use std::time::{Duration, Instant};

pub struct ProgressBar {
    total: usize,
    current: usize,
    start_time: Instant,
    label: String,
    bar_width: usize,
    last_update: Instant,
    min_update_interval: Duration,
    spinner_frames: Vec<&'static str>,
    spinner_index: usize,
    is_spinner: bool,
    finished: bool,
    current_file: Option<String>,
    details: Vec<String>,
}

pub struct StepProgress {
    steps: Vec<ProgressStep>,
    current_index: usize,
    start_time: Instant,
    sub_steps: Vec<String>,
    current_sub_step: usize,
    current_file: Option<String>,
    files_processed: Vec<String>,
    show_sub_steps: bool,
}

pub struct ProgressStep {
    pub name: String,
    pub description: String,
    pub status: StepStatus,
    pub duration_ms: Option<u64>,
    pub sub_steps: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

impl ProgressBar {
    pub fn new(total: usize, label: &str) -> Self {
        Self {
            total,
            current: 0,
            start_time: Instant::now(),
            label: label.to_string(),
            bar_width: 40,
            last_update: Instant::now().checked_sub(Duration::from_secs(1)).unwrap(),
            min_update_interval: Duration::from_millis(50),
            spinner_frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            spinner_index: 0,
            is_spinner: false,
            finished: false,
            current_file: None,
            details: Vec::new(),
        }
    }

    pub fn spinner(label: &str) -> Self {
        let mut bar = Self::new(0, label);
        bar.is_spinner = true;
        bar
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
        self.force_render();
    }

    pub fn set_current_file(&mut self, file: &str) {
        self.current_file = Some(file.to_string());
        self.render_if_needed();
    }

    pub fn inc(&mut self) {
        self.current += 1;
        self.render_if_needed();
    }

    pub fn set(&mut self, current: usize) {
        self.current = current;
        self.render_if_needed();
    }

    pub fn add_detail(&mut self, detail: &str) {
        self.details.push(detail.to_string());
        self.force_render();
    }

    fn render_if_needed(&mut self) {
        if self.last_update.elapsed() >= self.min_update_interval {
            self.force_render();
        }
    }

    fn force_render(&mut self) {
        if self.finished {
            return;
        }

        self.spinner_index = (self.spinner_index + 1) % self.spinner_frames.len();

        if self.is_spinner {
            self.render_spinner();
        } else {
            self.render_bar();
        }

        self.last_update = Instant::now();
    }

    fn render_bar(&mut self) {
        let progress = if self.total > 0 {
            self.current as f64 / self.total as f64
        } else {
            0.0
        };

        let filled = (progress * self.bar_width as f64) as usize;
        let empty = self.bar_width - filled;

        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        let percent = (progress * 100.0) as usize;
        let elapsed = self.start_time.elapsed().as_secs_f64();

        let eta = if self.current > 0 && self.total > 0 {
            let remaining = self.total - self.current;
            let per_item = self.start_time.elapsed().as_secs_f64() / self.current as f64;
            per_item * remaining as f64
        } else {
            0.0
        };

        let file_line = if let Some(ref f) = self.current_file {
            let short = if f.len() > 50 {
                format!("...{}", &f[f.len()-47..])
            } else {
                f.clone()
            };
            format!("\n  📄 {short}")
        } else {
            String::new()
        };

        print!(
            "\r  ⏳ {} [{:3}%] [{:<40}] {}/{} | {:.1}s / ~{:.0}s{}",
            self.label,
            percent,
            bar,
            self.current,
            self.total,
            elapsed,
            eta,
            file_line
        );
        std::io::stdout().flush().ok();
    }

    fn render_spinner(&mut self) {
        let frame = self.spinner_frames[self.spinner_index];
        let elapsed = self.start_time.elapsed().as_secs_f64();

        let file_line = if let Some(ref f) = self.current_file {
            let short = if f.len() > 50 {
                format!("...{}", &f[f.len()-47..])
            } else {
                f.clone()
            };
            format!("\n  📄 {short}")
        } else {
            String::new()
        };

        print!(
            "\r  {} {} ({:.1}s){}\n",
            frame,
            self.label,
            elapsed,
            file_line
        );
        std::io::stdout().flush().ok();
    }

    pub fn finish(&mut self) {
        self.finished = true;
        let elapsed = self.start_time.elapsed().as_secs_f64();

        if self.is_spinner {
            println!("\r  ✅ {} 完成 ({:.1}s)", self.label, elapsed);
        } else {
            println!(
                "\r  ✅ {} 完成 [{}/{}, {:.1}s]                          ",
                self.label,
                self.current,
                self.total,
                elapsed
            );
        }
    }

    pub fn fail(&mut self, error: &str) {
        self.finished = true;
        let elapsed = self.start_time.elapsed().as_secs_f64();
        println!("\r  ❌ {} 失败 ({:.1}s)\n     {}", self.label, elapsed, error);
    }

    pub fn finish_with_message(&mut self, message: &str) {
        self.finished = true;
        let elapsed = self.start_time.elapsed().as_secs_f64();
        println!("\r  ✅ {} 完成 ({:.1}s) - {}", self.label, elapsed, message);
    }
}

impl StepProgress {
    pub fn new<S: Into<String>>(steps: Vec<(S, S)>) -> Self {
        let steps = steps
            .into_iter()
            .map(|(name, description)| ProgressStep {
                name: name.into(),
                description: description.into(),
                status: StepStatus::Pending,
                duration_ms: None,
                sub_steps: Vec::new(),
                files: Vec::new(),
            })
            .collect();

        Self {
            steps,
            current_index: 0,
            start_time: Instant::now(),
            sub_steps: Vec::new(),
            current_sub_step: 0,
            current_file: None,
            files_processed: Vec::new(),
            show_sub_steps: true,
        }
    }

    pub fn with_sub_steps(mut self, show: bool) -> Self {
        self.show_sub_steps = show;
        self
    }

    pub fn set_sub_steps<S: Into<String>>(&mut self, sub_steps: Vec<S>) {
        if self.current_index < self.steps.len() {
            self.steps[self.current_index].sub_steps = sub_steps.into_iter().map(std::convert::Into::into).collect();
        }
        self.sub_steps = Vec::new();
        self.current_sub_step = 0;
        self.render();
    }

    pub fn next_sub_step<S: Into<String>>(&mut self, sub_step: S) {
        self.sub_steps.push(sub_step.into());
        self.current_sub_step = self.sub_steps.len();
        if self.current_index < self.steps.len() {
            self.steps[self.current_index].sub_steps = self.sub_steps.clone();
        }
        self.render();
    }

    pub fn set_current_file(&mut self, file: &str) {
        self.current_file = Some(file.to_string());
        if self.current_index < self.steps.len() {
            let step = &mut self.steps[self.current_index];
            if !step.files.contains(&file.to_string()) {
                step.files.push(file.to_string());
            }
        }
        self.files_processed.push(file.to_string());
        self.render();
    }

    pub fn add_file(&mut self, file: &str) {
        if self.current_index < self.steps.len() {
            let step = &mut self.steps[self.current_index];
            if !step.files.contains(&file.to_string()) {
                step.files.push(file.to_string());
            }
        }
        self.render();
    }

    pub fn start_next(&mut self) -> Option<&ProgressStep> {
        if self.current_index < self.steps.len() {
            self.steps[self.current_index].status = StepStatus::Running;
            self.steps[self.current_index].duration_ms = Some(0);
            self.sub_steps = Vec::new();
            self.current_sub_step = 0;
            self.current_file = None;
            self.files_processed = Vec::new();
            self.render();
            Some(&self.steps[self.current_index])
        } else {
            None
        }
    }

    pub fn complete_current(&mut self) {
        if self.current_index < self.steps.len() {
            let step = &mut self.steps[self.current_index];
            step.status = StepStatus::Success;
            step.duration_ms = Some(
                step.duration_ms.unwrap_or(0) + self.start_time.elapsed().as_millis() as u64,
            );
            self.current_index += 1;
            self.current_file = None;
            self.render();
        }
    }

    pub fn fail_current(&mut self, error: &str) {
        if self.current_index < self.steps.len() {
            let step = &mut self.steps[self.current_index];
            step.status = StepStatus::Failed;
            step.duration_ms = Some(
                step.duration_ms.unwrap_or(0) + self.start_time.elapsed().as_millis() as u64,
            );
            self.render();
            println!("   └─ ❌ {error}");
        }
    }

    pub fn skip_current(&mut self) {
        if self.current_index < self.steps.len() {
            self.steps[self.current_index].status = StepStatus::Skipped;
            self.current_index += 1;
            self.render();
        }
    }

    pub fn current_step_name(&self) -> Option<&str> {
        if self.current_index < self.steps.len() {
            Some(&self.steps[self.current_index].name)
        } else {
            None
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_index >= self.steps.len()
    }

    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }

    pub fn completed_steps(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| s.status == StepStatus::Success || s.status == StepStatus::Skipped)
            .count()
    }

    fn render(&self) {
        let width = 60;
        let total = self.steps.len();
        let completed = self.completed_steps();
        let pct = if total > 0 { completed * 100 / total } else { 100 };

        println!("\n┌{}┐", "─".repeat(width));
        println!("│  📊 执行进度  [{:>3}%]  ({}/{}) {:>width$}", 
            pct, completed, total, "", width = width - 25);
        println!("├{}┤", "─".repeat(width));

        for step in &self.steps {
            let icon = match step.status {
                StepStatus::Pending => "○",
                StepStatus::Running => "●",
                StepStatus::Success => "✓",
                StepStatus::Failed => "✗",
                StepStatus::Skipped => "⊘",
            };

            let status_icon = match step.status {
                StepStatus::Running => "🔄",
                StepStatus::Success => "✅",
                StepStatus::Failed => "❌",
                StepStatus::Skipped => "⏭️",
                StepStatus::Pending => "⏳",
            };

            let duration_str = if let Some(ms) = step.duration_ms {
                if ms > 0 {
                    format!(" ({:.1}s)", ms as f64 / 1000.0)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            println!(
                "│ {} {} {}{}{}",
                status_icon,
                icon,
                step.name,
                duration_str,
                " ".repeat(width.saturating_sub(step.name.len() + 12 + duration_str.len()))
            );

            if step.status == StepStatus::Running && self.show_sub_steps {
                if !step.sub_steps.is_empty() {
                    for (j, sub) in step.sub_steps.iter().enumerate() {
                        let sub_icon = if j < self.current_sub_step.saturating_sub(1) {
                            "  ✓"
                        } else if j == self.current_sub_step.saturating_sub(1) {
                            "  →"
                        } else {
                            "  ·"
                        };
                        println!("│    {sub_icon} {sub}");
                    }
                }

                if let Some(ref f) = self.current_file {
                    let short = if f.len() > width - 10 {
                        format!("...{}", &f[f.len() - (width - 13)..])
                    } else {
                        f.clone()
                    };
                    println!("│    📄 {short}");
                }

                if !step.files.is_empty() && step.files.len() > 1 {
                    println!("│    📁 已处理 {} 个文件", step.files.len());
                }
            }

            if step.status == StepStatus::Success && !step.files.is_empty() {
                let file_count = step.files.len();
                if file_count > 0 {
                    println!("│    📁 {file_count} 个文件");
                }
            }
        }

        println!("└{}┘", "─".repeat(width));
    }

    pub fn render_summary(&self) {
        let total_time = self.start_time.elapsed().as_secs_f64();
        let success_count = self.steps.iter().filter(|s| s.status == StepStatus::Success).count();
        let failed_count = self.steps.iter().filter(|s| s.status == StepStatus::Failed).count();
        let skipped_count = self.steps.iter().filter(|s| s.status == StepStatus::Skipped).count();

        let total_files: usize = self.steps.iter().map(|s| s.files.len()).sum();

        println!("\n╔══════════════════════════════════════════════════════╗");
        println!("║                  📋 执行总结                          ║");
        println!("╠══════════════════════════════════════════════════════╣");
        println!("║  总耗时: {total_time:<41.2}s ║");
        println!("║  步骤:  ✅ {}  |  ❌ {}  |  ⏭️ {}  / 共 {} 个     ║", 
            success_count, failed_count, skipped_count, self.steps.len());
        println!("║  成功率: {:<41.1}% ║", if self.steps.is_empty() {
            0.0
        } else {
            success_count as f64 / self.steps.len() as f64 * 100.0
        });
        if total_files > 0 {
            println!("║  处理文件: {total_files:<38} 个 ║");
        }

        let mut slowest: Vec<(&ProgressStep, u64)> = self.steps.iter()
            .filter(|s| s.duration_ms.unwrap_or(0) > 0)
            .map(|s| (s, s.duration_ms.unwrap_or(0)))
            .collect();
        slowest.sort_by_key(|s| std::cmp::Reverse(s.1));

        if !slowest.is_empty() {
            println!("╠══════════════════════════════════════════════════════╣");
            println!("║  耗时最长的步骤:                                     ║");
            for (step, ms) in slowest.iter().take(3) {
                println!("║   · {:<30} {:>10.2}s     ║", step.name, *ms as f64 / 1000.0);
            }
        }

        println!("╚══════════════════════════════════════════════════════╝");
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if !self.finished {
            self.finish();
        }
    }
}
