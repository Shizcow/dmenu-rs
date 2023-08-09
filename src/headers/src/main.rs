use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn main() {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))
        .expect("Could not grab stdout!");
    print!("SUCCESS ");
    stdout
        .set_color(ColorSpec::new().set_bold(true))
        .expect("Could not grab stdout!");
    println!("Headers generated successfully");
}
