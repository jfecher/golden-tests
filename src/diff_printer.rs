use difference::{ Changeset, Difference };
use colored::Colorize;
use std::fmt::{ Formatter, Display, Error };

pub struct DiffPrinter(pub Changeset);

fn print_line_number<F, D: Display>(current_line: Option<usize>,
        f: &mut Formatter, colorizer: &F) -> Result<Option<usize>, Error>
    where F: Fn(&str) -> D,
{
    let line_number = current_line.as_ref().map_or(" ".to_string(), |line| line.to_string());
    let line_number_string = format!("{:>3}| ", line_number);
    write!(f, "{}", colorizer(&line_number_string))?;
    Ok(current_line.map(|x| x + 1))
}

fn fmt_lines<F, D: Display>(lines: &str, mut current_line: Option<usize>,
        f: &mut Formatter, colorizer: F) -> Result<usize, Error>
    where F: Fn(&str) -> D,
{
    current_line = print_line_number(current_line, f, &colorizer)?;

    for character in lines.chars() {
        if character == '\r' {
            // Do nothing
        } else if character == '\n' {
            writeln!(f, "")?;
            current_line = print_line_number(current_line, f, &colorizer)?;
        } else {
            write!(f, "{}", colorizer(&character.to_string()))?;
        }
    }

    writeln!(f, "")?;
    Ok(current_line.unwrap_or(0))
}

impl Display for DiffPrinter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut line = 1;
        for i in 0 .. self.0.diffs.len() {
            match &self.0.diffs[i] {
                Difference::Same(lines) => {
                    line = fmt_lines(lines, Some(line), f, |x| x.normal())?;
                },
                Difference::Add(lines) => {
                    // Don't show/increment the line number if the previous change was a Removal
                    if i > 0 && matches!(self.0.diffs[i - 1], Difference::Rem(..)) {
                        fmt_lines(lines, None, f, |x| x.green())?;
                    } else {
                        line = fmt_lines(lines, Some(line), f, |x| x.green())?;
                    }
                },
                Difference::Rem(lines) => {
                    // Don't show/increment the line number unless the next change is an Addition
                    if i < self.0.diffs.len() - 1 && matches!(self.0.diffs[i + 1], Difference::Add(..)) {
                        line = fmt_lines(lines, Some(line), f, |x| x.red())?;
                    } else {
                        fmt_lines(lines, None, f, |x| x.red())?;
                    }
                },
            }
        }
        Ok(())
    }
}
