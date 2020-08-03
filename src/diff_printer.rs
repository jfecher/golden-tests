use difference::{ Changeset, Difference };
use colored::{
    Color::{Green, Red},
    Colorize,
};
use std::fmt::{ Formatter, Display, Error };

pub struct DiffPrinter(pub Changeset);

fn print_line_number<F, D: Display>(current_line: Option<usize>,
        f: &mut Formatter, colorizer: &F) -> Result<Option<usize>, Error>
    where F: Fn(bool, &str) -> D,
{
    let line_number = current_line.as_ref().map_or(" ".to_string(), |line| line.to_string());
    let line_number_string = format!("{:>3}| ", line_number);
    write!(f, "{}", colorizer(false, &line_number_string))?;
    Ok(current_line.map(|x| x + 1))
}

fn fmt_lines<F, D: Display>(lines: &str, mut current_line: Option<usize>,
        f: &mut Formatter, colorizer: F) -> Result<usize, Error>
    where F: Fn(bool, &str) -> D,
{
    current_line = print_line_number(current_line, f, &colorizer)?;
    let len = lines.len().saturating_sub(1);

    for (idx, character) in lines.chars().enumerate() {
        if character == '\r' {
            // Do nothing
        } else if character == '\n' {
            writeln!(f, "")?;
            current_line = print_line_number(current_line, f, &colorizer)?;
        } else {
            write!(f, "{}", colorizer(idx == len && character.is_whitespace(), &character.to_string()))?;
        }
    }

    writeln!(f, "")?;
    Ok(current_line.unwrap_or(0))
}

impl Display for DiffPrinter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        macro_rules! color {
            ($b:expr, $s:expr, $c:expr) => {
                if $b {
                    $s.on_color($c)
                } else {
                    $s.color($c)
                }
            };
        }
        let mut line = 1;
        for i in 0 .. self.0.diffs.len() {
            match &self.0.diffs[i] {
                Difference::Same(lines) => {
                    line = fmt_lines(lines, Some(line), f, |_, x| x.normal())?;
                },
                Difference::Add(lines) => {
                    // Don't show/increment the line number if the previous change was a Removal
                    if i > 0 && matches!(self.0.diffs[i - 1], Difference::Rem(..)) {
                        fmt_lines(lines, None, f, |b, x| color!(b, x, Green))?;
                    } else {
                        line = fmt_lines(lines, Some(line), f, |b, x| color!(b, x, Green))?;
                    }
                },
                Difference::Rem(lines) => {
                    // Don't show/increment the line number unless the next change is an Addition
                    if i < self.0.diffs.len() - 1 && matches!(self.0.diffs[i + 1], Difference::Add(..)) {
                        line = fmt_lines(lines, Some(line), f, |b, x| color!(b, x, Red))?;
                    } else {
                        fmt_lines(lines, None, f, |b, x| color!(b, x, Red))?;
                    }
                },
            }
        }
        Ok(())
    }
}