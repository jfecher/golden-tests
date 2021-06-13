use similar::{ChangeTag, TextDiff, DiffOp, Change};
use colored::{Color, Colorize, ColoredString};
use std::fmt::{ Formatter, Display, Error };

pub struct DiffPrinter<'a>(pub TextDiff<'a, 'a, 'a, str>);

fn print_line_number(index: Option<usize>, f: &mut Formatter, colorizer: Colorizer) -> Result<(), Error>  {
    let line_number = index.map_or_else(String::new, |line| (line + 1).to_string());
    let line_number_string = format!("{:>3}| ", line_number);
    
    write!(f, "{}", colorizer.color(false, &line_number_string))
}

fn fmt_line(f: &mut Formatter, index: Option<usize>, change: Change<str>) -> Result<(), Error> {
    let colorizer = match change.tag() {
        ChangeTag::Delete => Colorizer::colored(Color::Red),
        ChangeTag::Equal => Colorizer::normal(),
        ChangeTag::Insert => Colorizer::colored(Color::Green),
    };
    print_line_number(index, f, colorizer)?;

    writeln!(f, "{}", colorizer.color(false, change.to_string().strip_suffix('\n').unwrap()))
}

#[derive(Copy, Clone)]
struct Colorizer {
    color: Color,
    pass: bool,
}

impl Colorizer {
    const fn colored(color: Color) -> Colorizer {
        Colorizer { color, pass: false }
    }

    const fn normal() -> Colorizer {
        Colorizer { color: Color::Black, pass: true }
    }

    fn color(&self, background: bool, character: &str) -> ColoredString {
        if self.pass {
            return character.normal()
        } else if background {
            character.on_color(self.color)
        } else {
            character.color(self.color)
        }
    }
}

impl Display for DiffPrinter<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for op in self.0.ops() {
            match op {
                DiffOp::Delete {..} | DiffOp::Equal {..} | DiffOp::Insert {..} => {
                    for change in self.0.iter_changes(op) {
                        fmt_line(f, change.new_index(), change)?;
                    }
                },
                DiffOp::Replace { new_index: start, new_len: len, .. } => {
                    let mut iter = self.0.iter_changes(op);
                    for (line, change) in (*start..).zip(iter.by_ref().take(*len)) {
                        fmt_line(f, Some(line), change)?;
                    }

                    for change in iter {
                        fmt_line(f, None, change)?;
                    }
                }
            }
        }
        Ok(())
    }
}
