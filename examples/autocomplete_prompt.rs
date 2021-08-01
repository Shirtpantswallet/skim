extern crate skim;
use skim::prelude::*;

pub fn main() {
    const DEFAULT_CMD: &str = "jq";
    const DEFAULT_ARG: &str = "-C";
    const DEFAULT_FILE: &str = "large-file.json";
    let item_reader_option = SkimItemReaderOption::default()
        .ansi(true)
        .build();

    let full_cmd = format!("{} {} '{{}}' {}",
                            DEFAULT_CMD,
                            DEFAULT_ARG,
                            DEFAULT_FILE);

    let cmd_collector = Rc::new(RefCell::new(SkimItemReader::new(item_reader_option)));

    let options = SkimOptionsBuilder::default()
        .color(Some(""))
        .cmd_collector(cmd_collector)
        // Allow persistent preview
        .no_clear_if_empty(true)
        .multi(false)
        .interactive(true)
        .cmd(Some(&full_cmd))
        .cmd_prompt(Some("jq filter > "))
        .bind(vec!["tab:autocomplete-begin"])
        .reverse(true)
        .build()
        .expect("failed to build SkimOptions");

    let final_cmd = Skim::run_with(&options, None)
        .map(|out| out.cmd)
        .unwrap_or_else(|| ".".to_string());

    let output = std::process::Command::new(DEFAULT_CMD)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .args(&vec![DEFAULT_ARG, &final_cmd, DEFAULT_FILE])
        .spawn()
        .expect("failed to spawn child process")
        .wait_with_output()
        .expect("failed to read stdout");

    let status = output.status;
    if status.success() {
        println!("{}", std::str::from_utf8(&output.stdout).unwrap().to_string());
    }
    else {
        println!("failed to run final command");
    }
}
