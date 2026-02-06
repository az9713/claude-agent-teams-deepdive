// A sample Rust file for testing TODO detection

fn main() {
    // TODO: Add proper argument parsing
    let x = 42;

    // FIXME(alice): This calculation is wrong
    let y = x * 2;

    // HACK: Temporary workaround for issue #789
    println!("{}", y);

    let todo_string = "This has TODO in a string but it's not a comment";
}

// TODO(bob, #123, p:high): Implement error handling
fn placeholder() {}
