#![allow(unused)]

use anyhow::Result;
use mock_store::Store;
use serde::{Deserialize, Serialize};

pub const TICKET_TITLES: [&str; 5] = ["hello one", "hello two", "byebye all", "byebye all", "someting else"];

pub const PROJECT_NAMES: [&str; 3] = ["project AAA", "project BBB", "project ccc"];

pub fn seed_store() -> Result<Store> {
	let mut store = Store::new();

	// -- Tickets
	let mut id_seq = 100; // id starts at 100
	for title in TICKET_TITLES {
		store.insert(Ticket::new(id_seq, title))?;
		id_seq += 1;
	}

	// -- Tickets
	let mut id_seq = 100; // id starts at 100
	for name in PROJECT_NAMES {
		store.insert(Project::new(id_seq, name))?;
		id_seq += 1;
	}

	Ok(store)
}

// region:    --- Ticket
#[derive(Debug, Deserialize, Serialize)]
pub struct Ticket {
	pub id: u64,
	pub title: String,
}

impl Ticket {
	pub fn new(id: u64, title: impl Into<String>) -> Self {
		Self {
			id,
			title: title.into(),
		}
	}
}

// endregion: --- Ticket

// region:    --- Project
#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
	pub id: u64,
	pub name: String,
}

impl Project {
	pub fn new(id: u64, name: impl Into<String>) -> Self {
		Self { id, name: name.into() }
	}
}
// endregion: --- Project
