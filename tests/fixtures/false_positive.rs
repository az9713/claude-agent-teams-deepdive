// This file tests false positive rejection
// Only TODOs inside comments should be detected

fn main() {
    let todo_string = "TODO: this is in a string";
    let fixme_var = "FIXME not a real fixme";
    let hack_mode = "HACK in string literal";

    println!("TODO should not be detected here");
    println!("FIXME also not here");

    // TODO: This IS a real TODO in a comment
}

fn another() {
    let msg = format!("Please fix the BUG in module");
    let x = "XXX content";
}
