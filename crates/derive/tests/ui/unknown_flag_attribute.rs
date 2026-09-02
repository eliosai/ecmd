use ecmd::Command;

#[derive(Command)]
#[command(name = "t")]
struct T {
    #[flag(short = 'v', loud)]
    verbose: bool,
}

fn main() {}
