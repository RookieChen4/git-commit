use git_message;
use std::io;
use std::process::Command;

fn main() {
    let mut git = git_message::CommitMessage::new();

    loop {
        println!("{}", git.status.content());
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        git.add_msg(input);
        if let git_message::MessageType::Subject(_) = git.status {
            break;
        }
        git.next()
    }
    println!("git content{:?}", git.content());
    let output = Command::new("git")
            .args(["commit", "-m", &git.content()])
            .output()
            .expect("failed to execute process");
}
