#![allow(unused)]

use anyhow::Result;
use mock_store::Store;
use modql::filter::{FilterGroup, FilterNode, OpValInt64, OpValString};
use serde::{Deserialize, Serialize};

// Types need to implement Serialize/Deserialize from serde (as the Value is raw data)
#[derive(Debug, Deserialize, Serialize)]
struct Ticket {
	id: u64,
	title: String,
}

fn main() -> Result<()> {
	// -- Store are Send + Sync (backed by Arc/Mutex).
	let store = Store::new();

	// -- Insert the objects.
	store.insert(Ticket {
		id: 1,
		title: "Ticket AAA".to_string(),
	})?;
	store.insert(Ticket {
		id: 1,
		title: "Ticket BBB".to_string(),
	})?;

	// -- List all tickets (no filter).
	let all_tickets = store.list::<Ticket>(None)?;
	// [Ticket { id: 1, title: "Ticket AAA" }, Ticket { id: 1, title: "Ticket BBB" }]
	println!("{:<20}: {all_tickets:?}", "all tickets");

	// -- List with filter (using modql: https://github.com/jeremychone/rust-modql)
	let filter: FilterGroup = vec![("title", OpValString::Contains("AA".to_string())).into()].into();
	let double_a_tickets = store.list::<Ticket>(filter)?;
	// [Ticket { id: 1, title: "Ticket AAA" }]
	println!("{:<20}: {double_a_tickets:?}", "double_a_tickets");

	// -- Update with filter.
	let filter: FilterGroup = vec![("title", OpValString::Contains("BB".to_string())).into()].into();
	let count = store.update::<Ticket, _>(filter, |mut ticket| {
		ticket.title = "TICKET BB - UPDATE".to_string();
		ticket
	})?;
	// 1
	println!("{:<20}: {count:?}", "tickets updated");

	// -- List all tickets again.
	let all_tickets = store.list::<Ticket>(None)?;
	// [Ticket { id: 1, title: "Ticket AAA" }, Ticket { id: 1, title: "TICKET BB - UPDATE" }]
	println!("{:<20}: {all_tickets:?}", "all tickets");

	// -- Delete is: store.delete::<Ticket>(filter)?:

	Ok(())
}
