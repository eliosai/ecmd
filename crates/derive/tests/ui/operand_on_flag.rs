use ecmd::Command;

#[derive(Command)]
#[command(name = "t")]
struct T {
    #[flag(short = 'v')]
    #[operand(hide)]
    verbose: bool,
}

fn main() {}
