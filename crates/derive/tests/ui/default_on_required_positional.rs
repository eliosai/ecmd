use ecmd::Command;

#[derive(Command)]
#[command(name = "t")]
struct T {
    #[operand(default = "x")]
    dir: String,
}

fn main() {}
