use crate::types::line::Line;
use anyhow::{anyhow, Result};
use std::cmp;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub enum Typing {
    BeforeStart(State),
    Running(State),
    Finish(State),
}

#[derive(Clone, Debug)]
pub struct State {
    current_index: usize,
    display_lines: usize,
    end_time: Option<std::time::Instant>,
    is_error: bool,
    lines: Vec<Line>,
    remaining_time: Duration,
    start_time: Option<std::time::Instant>,
    typed: usize,
    typo: usize,
}

impl Typing {
    pub fn new(text: &str, remaining_time: Duration, display_lines: usize) -> Result<Self> {
        if text.is_empty() {
            Err(anyhow!("text is empty"))
        } else {
            let lines: Vec<Line> = text
                .split('\n')
                .enumerate()
                .map(|(i, v)| Line::new(i + 1, v))
                .collect();
            Ok(Typing::BeforeStart(State {
                current_index: 0,
                lines: lines.clone(),
                start_time: None,
                end_time: None,
                remaining_time,
                typed: 0,
                typo: 0,
                is_error: false,
                display_lines,
            }))
        }
    }

    pub fn restart(&self, text: &str, remaining_time: Duration) -> Self {
        match self {
            Typing::Finish(s) => Typing::BeforeStart(State {
                current_index: 0,
                lines: Typing::to_lines(text),
                start_time: None,
                end_time: None,
                remaining_time,
                typed: 0,
                typo: 0,
                ..s.clone()
            }),
            Typing::Running(s) => Typing::Running(s.clone()),
            Typing::BeforeStart(s) => Typing::BeforeStart(s.clone()),
        }
    }

    pub fn start(&self) -> Self {
        match self {
            Typing::BeforeStart(s) => Typing::Running(State {
                start_time: Some(Instant::now()),
                ..s.clone()
            }),
            Typing::Running(t) => Typing::Running(t.clone()),
            Typing::Finish(t) => Typing::Finish(t.clone()),
        }
    }

    pub fn finish(&self) -> Self {
        match self.clone() {
            Typing::Running(t) => Typing::Finish(State {
                end_time: Some(Instant::now()),
                ..t.clone()
            }),
            Typing::BeforeStart(lines) => Typing::BeforeStart(lines),
            Typing::Finish(t) => Typing::Finish(t),
        }
    }

    pub fn input(&self, c: char) -> Self {
        match self {
            Typing::Running(t) => {
                let current_line = t.current();
                let entered = current_line.input(c);
                let mut lines = t.lines.clone();

                if entered {
                    let next = current_line.next();

                    if next.is_entered() {
                        self.next()
                    } else {
                        lines[t.current_index] = next;
                        Typing::Running(State {
                            lines,
                            typed: t.typed + 1,
                            is_error: false,
                            ..t.clone()
                        })
                    }
                } else {
                    Typing::Running(State {
                        lines,
                        typed: t.typed,
                        typo: t.typo + 1,
                        is_error: true,
                        ..t.clone()
                    })
                }
            }
            Typing::BeforeStart(t) => Typing::BeforeStart(t.clone()),
            Typing::Finish(t) => Typing::Finish(t.clone()),
        }
    }

    pub fn next(&self) -> Self {
        match self.clone() {
            Typing::Running(t) => {
                if t.current_index + 1 < t.lines.len() {
                    Typing::Running(State {
                        current_index: t.current_index + 1,
                        lines: t.lines.clone(),
                        ..t.clone()
                    })
                } else {
                    self.finish()
                }
            }
            Typing::Finish(t) => Typing::Finish(t.clone()),
            Typing::BeforeStart(t) => Typing::BeforeStart(t.clone()),
        }
    }

    pub fn display_lines(&self) -> Vec<Line> {
        match self.clone() {
            Typing::Running(t) => t.display_lines(),
            Typing::Finish(t) => t.display_lines(),
            Typing::BeforeStart(t) => t.display_lines(),
        }
    }

    pub fn is_finish(&self) -> bool {
        matches!(self, Typing::Finish(_))
    }

    pub fn is_before_start(&self) -> bool {
        matches!(self, Typing::BeforeStart(_))
    }

    pub fn wpm(&self) -> usize {
        match self {
            Typing::Running(s) => s.wpm(),
            Typing::Finish(s) => s.wpm(),
            _ => 0,
        }
    }

    pub fn acc(&self) -> usize {
        match self {
            Typing::Running(s) => s.acc(),
            Typing::Finish(s) => s.acc(),
            _ => 0,
        }
    }

    pub fn typed(&self) -> usize {
        match self {
            Typing::Running(s) => s.typed,
            Typing::Finish(s) => s.typed,
            _ => 0,
        }
    }

    pub fn typo(&self) -> usize {
        match self {
            Typing::Running(s) => s.typo,
            Typing::Finish(s) => s.typo,
            _ => 0,
        }
    }

    pub fn tick(&self) -> Self {
        match self {
            Typing::Running(t) => {
                if t.remaining_time == Duration::from_secs(0) {
                    self.finish()
                } else {
                    Typing::Running(State {
                        remaining_time: if t.remaining_time == Duration::from_secs(0)
                            || t.remaining_time - Duration::from_secs(1) <= Duration::from_secs(1)
                        {
                            Duration::from_secs(0)
                        } else {
                            t.remaining_time - Duration::from_secs(1)
                        },
                        ..t.clone()
                    })
                }
            }
            Typing::Finish(t) => Typing::Finish(t.clone()),
            Typing::BeforeStart(t) => Typing::BeforeStart(t.clone()),
        }
    }

    pub fn get_remaining_time(&self) -> usize {
        match self.clone() {
            Typing::Running(t) => t.remaining_time.as_secs() as usize,
            Typing::Finish(t) => t.remaining_time.as_secs() as usize,
            Typing::BeforeStart(t) => t.remaining_time.as_secs() as usize,
        }
    }

    pub fn update_remaining_time(&self, time: Duration) -> Self {
        match self.clone() {
            Typing::Running(mut t) => {
                t.remaining_time = time;
                Typing::Running(t)
            }
            Typing::Finish(mut t) => {
                t.remaining_time = time;
                Typing::Finish(t)
            }
            Typing::BeforeStart(mut t) => {
                t.remaining_time = time;
                Typing::BeforeStart(t)
            }
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Typing::Running(s) => s.is_error,
            Typing::Finish(_) => false,
            _ => false,
        }
    }

    pub fn current_line_index(&self) -> usize {
        match self {
            Typing::Running(s) => s.current_index,
            Typing::Finish(s) => s.current_index,
            _ => 0,
        }
    }

    fn to_lines(text: &str) -> Vec<Line> {
        text.split('\n')
            .enumerate()
            .map(|(i, v)| Line::new(i + 1, v))
            .collect()
    }
}

impl State {
    pub fn running_time(&self) -> Duration {
        self.end_time
            .unwrap_or(Instant::now())
            .duration_since(self.start_time.unwrap_or(Instant::now()))
    }

    pub fn display_lines(&self) -> Vec<Line> {
        if self.lines.len() <= self.display_lines {
            self.lines.clone()
        } else {
            let start_index = if self.current_index > 0 {
                self.current_index - 1
            } else {
                0
            };
            self.lines[start_index..cmp::min(self.lines.len(), start_index + self.display_lines)]
                .to_vec()
        }
    }

    pub fn current(&self) -> Line {
        self.lines.get(self.current_index).unwrap().clone()
    }

    pub fn wpm(&self) -> usize {
        let sec = self.running_time().as_secs();
        let sec = usize::try_from(if sec > 0 { sec } else { 1 }).unwrap();
        ((self.typed + self.typo) / sec) * 60 / 5
    }

    pub fn acc(&self) -> usize {
        ((self.typed as f64 / (self.typed as f64 + self.typo as f64)) * 100.0).round() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_lines() {
        let typing = Typing::new("    line1\n  line2", Duration::from_secs(10), 10);
        let typing = typing.unwrap().start();
        assert_eq!(typing.display_lines().len(), 2);

        let typing = Typing::new("    line1\n  line2\n line3\n line4\n line5\n line6\n line7\n line8\n line9\n line10\n line11", Duration::from_secs(10), 10);
        let typing = typing.unwrap().start();
        assert_eq!(typing.display_lines().len(), 10);
    }

    #[test]
    fn next_line() {
        let typing = Typing::new("    line1\n  line2", Duration::from_secs(10), 10);
        let typing = typing.unwrap().start();
        let next = typing.next();

        match next {
            Typing::Running(line) => {
                assert_eq!(line.current().rest_text().unwrap(), "ine2");
            }
            _ => (),
        }
    }

    #[test]
    fn next_empty() {
        let typing = Typing::new("    line1\n  line2", Duration::from_secs(10), 10);
        let typing = typing.unwrap().start();
        let next = typing.next().next();
        assert!(next.is_finish());
    }

    #[test]
    fn empty_lines() {
        assert!(Typing::new("", Duration::from_secs(0), 10).is_err());
    }

    #[test]
    fn wpm() {
        let typing = Typing::new("    line1\n  line2", Duration::from_secs(10), 10);
        let typing = typing.unwrap().start();
        let typing = typing.input('l');
        let typing = typing.input('i');
        let typing = typing.input('n');
        let typing = typing.input('e');
        let typing = typing.finish();

        assert_eq!(typing.wpm(), 48);
    }

    #[test]
    fn acc() {
        let typing = Typing::new("    line1\n  line2", Duration::from_secs(10), 10);
        let typing = typing.unwrap().start();
        let typing = typing.input('l');
        let typing = typing.input('2');
        let typing = typing.input('i');
        let typing = typing.input('n');
        let typing = typing.input('e');
        let typing = typing.finish();

        assert_eq!(typing.acc(), 80);
    }
}
