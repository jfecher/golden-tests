use difference::{ Changeset, Difference };
use colored::{Color, Colorize, ColoredString};
use std::fmt::{ Formatter, Display, Error };

pub struct DiffPrinter(pub Changeset);

fn print_line_number(
    current_line: Option<usize>, 
    f: &mut Formatter, 
    colorizer: Colorizer
) -> Result<Option<usize>, Error>  {
    let line_number = current_line.as_ref().map_or(" ".to_string(), |line| line.to_string());
    let line_number_string = format!("{:>3}| ", line_number);
    write!(f, "{}", colorizer.color(false, &line_number_string))?;
    Ok(current_line.map(|x| x + 1))
}

fn fmt_lines(
    lines: &str,
    mut current_line: Option<usize>,
    f: &mut Formatter,
    colorizer: Colorizer
) -> Result<usize, Error> {
    current_line = print_line_number(current_line, f, colorizer)?;
    let len = lines.len().saturating_sub(1);

    for (idx, character) in lines.chars().enumerate() {
        if character == '\r' {
            // Do nothing
        } else if character == '\n' {
            writeln!(f, "")?;
            current_line = print_line_number(current_line, f, colorizer)?;
        } else {
            write!(f, "{}", colorizer.color(idx == len && character.is_whitespace(), &character.to_string()))?;
        }
    }

    writeln!(f, "")?;
    Ok(current_line.unwrap_or(0))
}

#[derive(Copy, Clone)]
pub struct Colorizer {
    color: Color,
    pass: bool,
}

impl Colorizer {
    pub const fn colored(color: Color) -> Colorizer {
        Colorizer { color, pass: false }
    }

    pub const fn normal() -> Colorizer {
        Colorizer { color: Color::Black, pass: true }
    }

    pub fn color(&self, background: bool, character: &str) -> ColoredString {
        if self.pass {
            return character.normal()
        } else if background {
            character.on_color(self.color)
        } else {
            character.color(self.color)
        }
    }
}

impl Display for DiffPrinter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {        
        let mut line = 1;

        for i in 0 .. self.0.diffs.len() {
            match &self.0.diffs[i] {
                Difference::Same(lines) => {
                    line = fmt_lines(lines, Some(line), f, Colorizer::normal())?;
                },
                Difference::Add(lines) => {
                    // Don't show/increment the line number if the previous change was a Removal
                    if i > 0 && matches!(self.0.diffs[i - 1], Difference::Rem(..)) {
                        fmt_lines(lines, None, f, Colorizer::colored(Color::Green))?;
                    } else {
                        line = fmt_lines(lines, Some(line), f, Colorizer::colored(Color::Green))?;
                    }
                },
                Difference::Rem(lines) => {
                    // Don't show/increment the line number unless the next change is an Addition
                    if i < self.0.diffs.len() - 1 && matches!(self.0.diffs[i + 1], Difference::Add(..)) {
                        line = fmt_lines(lines, Some(line), f, Colorizer::colored(Color::Red))?;
                    } else {
                        fmt_lines(lines, None, f, Colorizer::colored(Color::Red))?;
                    }
                },
            }
        }
        Ok(())
    }
}