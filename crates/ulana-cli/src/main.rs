//! `ulana`: render a view or check a tree from the command line.

use std::process::ExitCode;

use ulana::{Host, Limits, View};

const USAGE: &str = "\
usage:
  ulana render <view.luau> [--data <file.json>] [--event <id>[=<json>]]... [options]
  ulana check <tree.json> [options]
  ulana schema

render  runs a view, applies each --event in order (rendering before each),
        and prints the last frame as JSON.
check   reads a tree written as JSON and prints it checked.
schema  prints the JSON Schema of a frame.

options:
  --block <name>    accept custom blocks with this name (repeatable)
  --action <name>   accept host actions with this name (repeatable)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let (command, rest) = args.split_first().ok_or(USAGE)?;
    let mut file = None;
    let mut data = serde_json::Value::Null;
    let mut events = Vec::new();
    let mut host = Host::new();
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let mut value = || {
            it.next()
                .cloned()
                .ok_or(format!("{arg} needs a value\n\n{USAGE}"))
        };
        match arg.as_str() {
            "--block" => host = host.block(value()?),
            "--action" => host = host.action(value()?),
            "--data" => data = json_file(&value()?)?,
            "--event" => events.push(value()?),
            "-h" | "--help" => return Err(USAGE.into()),
            _ if file.is_none() && !arg.starts_with('-') => file = Some(arg.clone()),
            _ => return Err(format!("unexpected `{arg}`\n\n{USAGE}")),
        }
    }
    match command.as_str() {
        "schema" => Ok(pretty(&ulana::frame_schema())),
        "check" => {
            let tree = json_file(&file.ok_or(USAGE)?)?;
            let block = ulana::parse(tree, &host, &Limits::default()).map_err(|e| e.to_string())?;
            Ok(pretty(&block))
        }
        "render" => {
            let path = file.ok_or(USAGE)?;
            let source = std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
            let mut view =
                View::new(&source, host, Limits::default()).map_err(|e| e.to_string())?;
            let mut frame = render(&mut view, &data)?;
            for event in events {
                let (id, value) = match event.split_once('=') {
                    Some((id, json)) => (
                        id.to_string(),
                        serde_json::from_str(json).map_err(|e| format!("--event {event}: {e}"))?,
                    ),
                    None => (event.clone(), serde_json::Value::Null),
                };
                view.event(frame.version, &id, &value)
                    .map_err(|e| e.to_string())?;
                frame = render(&mut view, &data)?;
            }
            for line in view.take_logs() {
                eprintln!("print: {line}");
            }
            Ok(pretty(&frame))
        }
        _ => Err(USAGE.into()),
    }
}

fn render(view: &mut View, data: &serde_json::Value) -> Result<ulana::Frame, String> {
    view.render(data).map_err(|e| e.to_string())
}

fn json_file(path: &str) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))
}

fn pretty(value: &impl serde::Serialize) -> String {
    serde_json::to_string_pretty(value).expect("JSON")
}
