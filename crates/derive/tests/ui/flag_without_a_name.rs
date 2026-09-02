use ecmd::Command;

#[derive(Command)]
#[command(name = "t")]
struct T {
    #[flag(hide)]
    verbose: bool,
}

fn main() {}
