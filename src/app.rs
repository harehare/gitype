use crate::types::typing::Typing;
use anyhow::Result;
use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use std::time::Duration;

const SELECTABLE_TIME: [&usize; 4] = [&15, &30, &60, &120];

#[derive(Clone, Debug)]
pub struct App {
    pub time: Duration,
    pub typing: Typing,
    progress: TypingProgress,
    custom_time: Duration,
}

#[derive(Clone, Debug)]
pub struct TypingResult {
    pub wpm: usize,
    pub acc: usize,
    pub typed: usize,
    pub typo: usize,
    pub wpm_max: f64,
    pub wpm_plot: Vec<(f64, f64)>,
    pub acc_plot: Vec<(f64, f64)>,
}

impl App {
    pub fn new(text: &str, remaining_time: Duration, display_lines: usize) -> Result<App> {
        let text = App::filter_text(text);
        let typing = Typing::new(&text, remaining_time, display_lines)?;
        Ok(App {
            typing,
            time: remaining_time,
            custom_time: remaining_time,
            progress: TypingProgress::new(),
        })
    }

    pub fn result(&self) -> TypingResult {
        TypingResult {
            wpm: self.typing.wpm(),
            acc: self.typing.acc(),
            typed: self.typing.typed(),
            typo: self.typing.typo(),
            wpm_max: self.progress.wpm_max(),
            wpm_plot: self.progress.wpm_plot(),
            acc_plot: self.progress.acc_plot(),
        }
    }

    pub fn start(mut self) -> Self {
        match self.typing {
            Typing::BeforeStart(_) => {
                let typing = self.typing.update_remaining_time(self.time);
                self.typing = typing.start();
                self
            }
            _ => self,
        }
    }

    pub fn restart(mut self, text: &str) -> Self {
        let text = App::filter_text(text);
        self.typing = self.typing.restart(&text, self.time);
        self
    }

    pub fn finish(mut self) -> Self {
        self.typing = self.typing.finish();
        self
    }

    pub fn input(mut self, c: char) -> Self {
        self.typing = self.typing.input(c);
        self
    }

    pub fn backspace(mut self) -> Self {
        self.typing = self.typing.backspace();
        self
    }

    pub fn tick(mut self) -> Self {
        self.typing = self.typing.tick();
        self.progress = self.progress.add(self.typing.clone());
        self
    }

    pub fn selectable_time(&self) -> Vec<Duration> {
        let mut times = [
            SELECTABLE_TIME
                .map(|t| Duration::from_secs(*t as u64))
                .to_vec(),
            vec![self.custom_time],
        ]
        .concat();
        times.sort();
        times.dedup();
        times
    }

    pub fn next_time(mut self) -> Self {
        let time = self.time.as_secs();
        let custom_time = self.custom_time.as_secs();
        let time = Duration::from_secs(match time {
            15 => 30,
            30 => 60,
            60 => 120,
            120 => {
                if custom_time != 15 {
                    custom_time
                } else {
                    15
                }
            }
            _ => {
                if !(15..=120).contains(&custom_time) {
                    15
                } else {
                    custom_time
                }
            }
        });

        self.time = time;
        self
    }

    pub fn prev_time(mut self) -> Self {
        let time = self.time.as_secs();
        let custom_time = self.custom_time.as_secs();
        let time = Duration::from_secs(match time {
            15 => {
                if custom_time != 120 {
                    custom_time
                } else {
                    120
                }
            }
            30 => 15,
            60 => 30,
            120 => 60,
            _ => {
                if (15..=120).contains(&custom_time) {
                    120
                } else {
                    custom_time
                }
            }
        });

        self.time = time;
        self
    }

    pub fn elapsed_time(&self) -> Duration {
        self.time - Duration::from_secs(self.typing.get_remaining_time() as u64)
    }

    fn filter_text(text: &str) -> String {
        let text = ISO_8859_1.encode(text, EncoderTrap::Ignore).unwrap();
        ISO_8859_1
            .decode(&text, DecoderTrap::Strict)
            .unwrap()
            .replace('\t', "    ")
            .to_owned()
    }
}

#[derive(Clone, Debug)]
struct TypingProgress {
    wpm: Vec<usize>,
    acc: Vec<usize>,
}

impl TypingProgress {
    pub fn new() -> Self {
        TypingProgress {
            wpm: Vec::new(),
            acc: Vec::new(),
        }
    }

    pub fn add(mut self, typing: Typing) -> Self {
        self.wpm.push(typing.wpm());
        self.acc.push(typing.acc());
        self
    }

    pub fn wpm_max(&self) -> f64 {
        self.wpm.iter().fold(usize::MIN, |a, b| a.max(*b)) as f64
    }

    pub fn wpm_plot(&self) -> Vec<(f64, f64)> {
        let mut wpm: Vec<(f64, f64)> = self
            .wpm
            .iter()
            .enumerate()
            .map(|(i, wpm)| (i as f64, *wpm as f64))
            .collect();

        wpm.insert(0, (0.0, 0.0));
        wpm
    }

    pub fn acc_plot(&self) -> Vec<(f64, f64)> {
        let mut acc: Vec<(f64, f64)> = self
            .acc
            .iter()
            .enumerate()
            .map(|(i, acc)| (i as f64, *acc as f64))
            .collect();

        acc.insert(0, (0.0, 100.0));
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start() {
        let app = App::new("test", Duration::from_secs(10), 10).unwrap();
        assert_eq!(app.clone().start().typing.is_before_start(), false);
        assert_eq!(app.clone().start().typing.is_finish(), false);
    }

    #[test]
    fn restart() {
        let app = App::new("test", Duration::from_secs(10), 10).unwrap();
        assert_eq!(
            app.clone()
                .start()
                .finish()
                .restart("vv")
                .typing
                .is_before_start(),
            true
        );
    }

    #[test]
    fn selectable_time() {
        let app = App::new("test", Duration::from_secs(10), 10).unwrap();
        assert_eq!(app.clone().selectable_time().len(), 5);

        let app = App::new("test", Duration::from_secs(30), 10).unwrap();
        assert_eq!(app.clone().selectable_time().len(), 4);
    }

    #[test]
    fn tick() {
        let app = App::new("test", Duration::from_secs(10), 10).unwrap();
        assert_eq!(app.start().tick().typing.get_remaining_time(), 9);
    }

    #[test]
    fn elapsed_time() {
        let app = App::new("test", Duration::from_secs(10), 10).unwrap();
        assert_eq!(app.start().tick().elapsed_time(), Duration::from_secs(1));
    }

    #[test]
    fn next_time_less_then_15() {
        let app = App::new("test", Duration::from_secs(10), 10).unwrap();
        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(15));

        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(30));

        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(60));

        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(120));

        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(10));
    }

    #[test]
    fn next_time_greater_than_120() {
        let app = App::new("test", Duration::from_secs(240), 10).unwrap();
        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(15));

        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(30));

        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(60));

        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(120));

        let app = app.next_time();
        assert_eq!(app.time, Duration::from_secs(240));
    }

    #[test]
    fn prev_time_less_then_15() {
        let app = App::new("test", Duration::from_secs(10), 10).unwrap();
        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(120));

        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(60));

        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(30));

        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(15));

        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(10));
    }

    #[test]
    fn prev_time_greater_than_120() {
        let app = App::new("test", Duration::from_secs(240), 10).unwrap();
        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(120));

        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(60));

        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(30));

        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(15));

        let app = app.prev_time();
        assert_eq!(app.time, Duration::from_secs(240));
    }
}
