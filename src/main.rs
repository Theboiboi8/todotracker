use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

const STATE_MANIFEST_VERSION: usize = 1;

fn main() {
	let mut state = State::new();

	println!("Todo Tracker");

	while !state.exit {
		println!("Enter a command:");

		let mut buffer = String::new();
		std::io::stdin().read_line(&mut buffer).unwrap_or_default();
		buffer = buffer.trim_end().to_string();

		let command = Command::from(buffer);

		match command {
			Command::Add => {
				println!("Name of todo entry:");

				let mut name = String::new();
				std::io::stdin().read_line(&mut name).unwrap_or_default();
				name = name.trim_end().to_string();

				println!("Description of todo entry:");

				let mut description = String::new();
				std::io::stdin().read_line(&mut description).unwrap_or_default();
				description = description.trim_end().to_string();

				command.execute(&mut state, CommandState::add(name, description));
			}
			Command::Remove => {
				println!("Index of entry to remove:");

				let mut index = String::new();
				std::io::stdin().read_line(&mut index).unwrap_or_default();
				let index = index.trim_end().to_string().parse::<usize>().unwrap_or_else(
					|_| {
						eprintln!("No entry found at that index");
						usize::MAX
					}
				);

				command.execute(&mut state, CommandState::remove(index));
			}
			_ => command.execute(&mut state, CommandState::empty())
		}
	}
}

#[derive(Clone, Serialize, Deserialize)]
struct State {
	pub entries: Vec<TodoEntry>,
	pub exit: bool,
	pub manifest_version: usize,
}

struct CommandState {
	index: Option<usize>,
	name: Option<String>,
	description: Option<String>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Serialize, Deserialize)]
struct TodoEntry {
	pub name: String,
	pub description: String,
}

#[derive(EnumIter, Ord, PartialOrd, Eq, PartialEq)]
enum Command {
	Help,
	List,
	Add,
	Remove,
	Clear,
	Save,
	Load,
	Exit,
	Unknown,
}

impl State {
	fn new() -> Self {
		State {
			entries: Vec::<TodoEntry>::new(),
			exit: false,
			manifest_version: STATE_MANIFEST_VERSION,
		}
	}
}

impl TodoEntry {
	fn new(name: String, description: String) -> Self {
		TodoEntry {
			name,
			description
		}
	}
}

impl Command {
	pub fn key(&self) -> &str {
		match self {
			Command::Help => "help",
			Command::List => "list",
			Command::Add => "add",
			Command::Remove => "remove",
			Command::Clear => "clear",
			Command::Save => "save",
			Command::Load => "load",
			Command::Exit => "exit",
			Command::Unknown => unreachable!(),
		}
	}

	pub fn description(&self, ) -> &str {
		match self {
			Command::Help => "Displays a help message",
			Command::List => "Lists all todo entries",
			Command::Add => "Adds a new todo entry",
			Command::Remove => "Removes a todo entry by its index",
			Command::Clear => "Clears all todo entries",
			Command::Save => "Saves the current todo entries to a file",
			Command::Load => "Loads the todo entries from a file",
			Command::Exit => "Exits the program",
			Command::Unknown => unreachable!()
		}
	}

	#[allow(clippy::too_many_lines)]
	pub fn execute(self, state: &mut State, command_state: CommandState) {
		match self {
			Command::Help => {
				for command in Command::iter() {
					if command == Command::Unknown { break }
					println!("{command} ({}) : {}", command.key(), command.description());
				}
			}
			Command::List => {
				if state.entries.is_empty() {
					println!("Nothing to list");
				} else {
					for entry in &state.entries {
						println!(
							"{} - {}: {}",
							state.entries.binary_search(entry).unwrap_or_else(|_| {
								eprintln!("Failed to get index of entry!");
								usize::MAX
							}),
							entry.name,
							entry.description
						);
					}
				}
			}
			Command::Add => {
				if let (
					Some(name),
					Some(description)
				) = (
					command_state.name,
					command_state.description
				) {
					state.entries.push(TodoEntry::new(name, description));
				} else if cfg!(debug_assertions) {
					eprintln!("command_state.name and command_state.description \
					are required to be Some for Command::Add");
				}
			}
			Command::Remove => {
				if let Some(index) = command_state.index {
					if state.entries.get(index).is_some() {
						println!("Removed entry {}", state.entries.get(index).unwrap().name);
						state.entries.remove(index);
					} else {
						eprintln!("No todo entry found at index {index}");
					}
				} else if cfg!(debug_assertions) {
					eprintln!("command_state.index is required to be Some for Command::Remove");
				}
			}
			Command::Clear => {
				if state.entries.is_empty() {
					println!("Nothing to clear");
				} else {
					let entries_count = state.entries.len();
					state.entries.clear();
					println!(
						"{entries_count} {} cleared",
						if entries_count > 1 {
							"entries"
						} else {
							"entry"
						}
					);
				}
			}
			Command::Save => {
				if state.entries.is_empty() {
					println!("Nothing to save");
					return;
				}

				let data = ron::ser::to_string_pretty(
					state,
					ron::ser::PrettyConfig::default()
				).unwrap_or_else(|_| {
					eprintln!("Failed to save state to a file!");
					String::new()
				});

				std::fs::write("state.ron", data).unwrap_or_else(|_| {
					eprintln!("Failed to write state data to file!");
				});

				if PathBuf::from("state.ron").exists() {
					println!("Saved state data to state.ron");
				}
			}
			Command::Load => {
				let mut should_abort = false;

				if PathBuf::from("state.ron").exists() {
					let data = ron::from_str::<State>(
						&std::fs::read_to_string("state.ron").unwrap_or_else(|_| {
							eprintln!("Failed to read state data from file. \
							Are you sure it exists?");
							should_abort = true;
							String::new()
						})
					).unwrap_or_else(|_| {
						eprintln!("Failed to parse state data from file!");
						should_abort = true;
						State::new()
					});
					
					match data.manifest_version.cmp(&state.manifest_version) {
						Ordering::Less => {
							eprintln!("This save file has an old manifest version, \
							and may not load correctly");
						}
						Ordering::Greater => {
							eprintln!("This save file has been created with a newer version, \
							and may not load correctly");
						}
						Ordering::Equal => {}
					}

					if data.entries != state.entries && !state.entries.is_empty() {
						let mut valid = false;

						while !valid {
							println!("Override current entries? (y/n)");

							let mut buffer = String::new();
							std::io::stdin().read_line(&mut buffer).unwrap_or_default();
							let buffer = buffer.trim_end();

							match buffer {
								"y" | "Y" | "yes" | "Yes" | "YES" => {
									valid = true;
								},
								"n" | "N" | "no" | "No" | "NO" => {
									return;
								},
								_ => {
									valid = false;
									eprintln!("Unknown input");
								}
							}
						}
					}

					if should_abort {
						eprintln!("Due to one or more previous errors, \
						a state file will not be created");
						return;
					}

					state.entries = data.entries;
					println!("Loaded {} entries from state file", state.entries.len());
				} else {
					eprintln!("No state data file found at that location");
				}
			}
			Command::Exit => {
				if PathBuf::from("state.ron").exists() {
					let data = ron::from_str::<State>(
						&std::fs::read_to_string("state.ron").unwrap_or_else(|_| {
							eprintln!("Failed to read state data file");
							String::new()
						})
					).unwrap_or_else(|_| {
						eprintln!("Failed to parse state data from file!");
						State::new()
					});

					if state.entries != data.entries {
						let mut valid = false;

						while !valid {
							println!("A save file exists, but you have unsaved data. \
							Are you sure you want to quit? (y/n)");

							let mut buffer = String::new();
							std::io::stdin().read_line(&mut buffer).unwrap_or_default();
							let buffer = buffer.trim_end();

							match buffer {
								"y" | "Y" | "yes" | "Yes" | "YES" => {
									valid = true;
								},
								"n" | "N" | "no" | "No" | "NO" => {
									return;
								},
								_ => {
									valid = false;
									eprintln!("Unknown input");
								}
							}
						}
					}
				}

				state.exit = true;
			}
			Command::Unknown => {
				eprintln!("Unknown command");
			}
		}
	}
}

impl Display for Command {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Command::Help => write!(f, "Help"),
			Command::List => write!(f, "List"),
			Command::Add => write!(f, "Add"),
			Command::Remove => write!(f, "Remove"),
			Command::Clear => write!(f, "Clear"),
			Command::Save => write!(f, "Save"),
			Command::Load => write!(f, "Load"),
			Command::Exit => write!(f, "Exit"),
			Command::Unknown => write!(f, "Unknown Command"),
		}
	}
}

impl From<String> for Command {
	fn from(value: String) -> Self {
		let value = value.as_str();

		match value {
			"help" | "Help" | "HELP" => Command::Help,
			"list" | "List" | "LIST" => Command::List,
			"add" | "Add" | "ADD" => Command::Add,
			"remove" | "Remove" | "REMOVE" => Command::Remove,
			"clear" | "Clear" | "CLEAR" => Command::Clear,
			"save" | "Save" | "SAVE" => Command::Save,
			"load" | "Load" | "LOAD" => Command::Load,
			"exit" | "Exit" | "EXIT" => Command::Exit,
			_ => Command::Unknown
		}
	}
}

impl CommandState {
	fn empty() -> Self {
		CommandState {
			name: None,
			description: None,
			index: None
		}
	}

	fn add(name: String, description: String) -> Self {
		CommandState {
			name: Some(name),
			description: Some(description),
			index: None
		}
	}

	fn remove(index: usize) -> Self {
		CommandState {
			name: None,
			description: None,
			index: Some(index)
		}
	}
}
