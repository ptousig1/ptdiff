use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use similar::{ChangeTag, TextDiff};

fn main()
	{
	// `run` returns a `Result`; handle its error at the program boundary,
	// where it can be printed before exiting with a failure status.
	if let Err(message) = run()
		{
		eprintln!("{message}");
		process::exit(1);
		}
	}

// Returning `Result` lets `?` propagate errors while keeping `main` responsible
// for presenting them to the user.
fn run() -> Result<(), String>
	{
	// `args_os` preserves non-Unicode file paths; `args` would require UTF-8.
	let mut args = env::args_os();
	let program = args.next().unwrap_or_default();
	let Some(first_arg) = args.next() else { return Err(usage(&program)); };

	if first_arg == std::ffi::OsStr::new("--version")
		{
		if args.next().is_some()
			{
			return Err(usage(&program));
			}
		println!("ptdiff {}", env!("CARGO_PKG_VERSION"));
		println!("Copyright (C) 2026 Patrick Tousignant <ptousig@outlook.com>");
		println!("License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>");
		println!("This is free software: you are free to change and redistribute it.");
		println!("There is NO WARRANTY, to the extent permitted by law.");
		return Ok(());
		}

	let old_path = first_arg;
	let Some(new_path) = args.next() else { return Err(usage(&program)); };
	if args.next().is_some()
		{
		return Err(usage(&program));
		}

	// `map_err` converts the I/O error into our CLI's `String` error type;
	// the trailing `?` returns early if reading either file fails.
	let old_text = fs::read_to_string(&old_path)
		.map_err(|error| format!("Failed to read {}: {error}", Path::new(&old_path).display()))?;
	let new_text = fs::read_to_string(&new_path)
		.map_err(|error| format!("Failed to read {}: {error}", Path::new(&new_path).display()))?;

	let output = render_diff(&old_text, &new_text);
	// `lock` avoids repeatedly locking stdout, and `as_bytes` supplies the
	// byte slice required by `Write::write_all`.
	io::stdout()
		.lock()
		.write_all(output.as_bytes())
		.map_err(|error| format!("Failed to write output: {error}"))
	}

fn usage(program: &std::ffi::OsStr) -> String
	{
	// `format!` is a macro that builds and returns an owned `String`.
	format!(
		"Usage: {} <old-file> <new-file>\n       {} --version",
		Path::new(program).display(),
		Path::new(program).display())
	}

fn render_diff(old_text: &str, new_text: &str) -> String
	{
	// `&str` borrows the input text instead of copying it; `similar` yields
	// borrowed line changes that are collected into an owned `String`.
	let diff = TextDiff::from_lines(old_text, new_text);
	let mut output = String::new();

	for change in diff.iter_all_changes()
		{
		// Matching the enum makes each change kind explicit.
		let color = match change.tag()
			{
			ChangeTag::Equal => "",
			ChangeTag::Insert => "\x1b[43m",
			ChangeTag::Delete => "\x1b[41m",
			};
		output.push_str(color);
		let value = change.value();
		// Reset before the newline: a terminal may scroll while processing it,
		// which otherwise carries the colored background into the new screen row.
		let line_ending = if value.ends_with("\r\n") { 2 }
								else if value.ends_with('\n') { 1 }
								else { 0 };
		output.push_str(&value[..value.len() - line_ending]);
		if !color.is_empty()
			{
			output.push_str("\x1b[0m");
			}
		output.push_str(&value[value.len() - line_ending..]);
		}

	output
	}
