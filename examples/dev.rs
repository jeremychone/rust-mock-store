// #![allow(unused)]

use anyhow::Result;
use mock_store::Store;
use modql::filter::{FilterGroup, FilterNode, OpValInt64, OpValString};
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
	let store = Store::new();

	// -- Tickets
	store.insert(Ticket::new(1, "hello one"))?;
	store.insert(Ticket::new(2, "hello two"))?;
	store.insert(Ticket::new(3, "bybye all"))?;
	store.insert(Ticket::new(4, "bybye all"))?;
	store.insert(Ticket::new(5, "something else"))?;

	// -- Projects
	store.insert(Project::new(1, "Project AA"))?;
	store.insert(Project::new(1, "hello BB"))?;

	// -- Delete 2
	let delete_filter: FilterGroup = vec![("id", OpValInt64::Eq(2)).into()].into();
	let count = store.delete::<Ticket>(delete_filter)?;
	println!("->> delete count {count}");

	// -- Update "bybye all"
	let update_filter = FilterNode::from(("title", "bybye all"));
	let count = store.update::<Ticket, _>(update_filter, |mut t| {
		t.title = format!("hello ticket #{} !!!!", t.id);
		t
	})?;
	println!("->> delete count {count}");

	// -- List all with "hello"
	let filter: FilterGroup = vec![("title", OpValString::Contains("hello".to_string())).into()].into();
	let all = store.list::<Ticket>(filter)?;

	for item in all {
		println!("->> item: {item:?}");
	}

	Ok(())
}

// region:    --- Ticket
#[derive(Debug, Deserialize, Serialize)]
struct Ticket {
	id: u64,
	title: String,
}

impl Ticket {
	fn new(id: u64, title: impl Into<String>) -> Self {
		Self {
			id,
			title: title.into(),
		}
	}
}

// endregion: --- Ticket

// region:    --- Project
#[derive(Debug, Deserialize, Serialize)]
struct Project {
	id: u64,
	name: String,
}

impl Project {
	fn new(id: u64, name: impl Into<String>) -> Self {
		Self { id, name: name.into() }
	}
}
// endregion: --- Project
