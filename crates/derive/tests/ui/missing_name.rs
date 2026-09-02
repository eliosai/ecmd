use ecmd::Command;

#[derive(Command)]
#[command(style = "gnu")]
struct T {
    #[flag(short = 'v')]
    verbose: bool,
}

fn main() {}
