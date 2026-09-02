use ecmd::Command;

#[derive(Command)]
#[command(name = "t")]
struct T {
    #[operand(required)]
    dir: Option<String>,
}

fn main() {}
