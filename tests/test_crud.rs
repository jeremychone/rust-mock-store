use crate::common::{seed_store, Project, Ticket, PROJECT_NAMES, TICKET_TITLES};
use anyhow::Context;
use mock_store::Store;
use modql::filter::FilterNode;

mod common;

#[test]
fn test_crud_create_one() -> anyhow::Result<()> {
	let store = Store::new();

	store.insert(Ticket::new(1, "hello one"))?;

	let tickets = store.list::<Ticket>(None)?;

	assert_eq!("hello one", tickets[0].title);

	Ok(())
}

#[test]
fn test_crud_create_many_with_seq() -> anyhow::Result<()> {
	let store = Store::new();

	store.insert(Ticket::new(store.seq_next::<Ticket>(), "T-ONE"))?;
	store.insert(Ticket::new(store.seq_next::<Ticket>(), "T-TWO"))?;

	let tickets = store.list::<Ticket>(None)?;

	assert_eq!(1, tickets[0].id);
	assert_eq!("T-ONE", tickets[0].title);

	assert_eq!(2, tickets[1].id);
	assert_eq!("T-TWO", tickets[1].title);

	Ok(())
}

#[test]
fn test_crud_delete_one() -> anyhow::Result<()> {
	let store = seed_store()?;

	let count = store.delete::<Ticket>(FilterNode::from(("id", 101)))?;
	assert_eq!(1, count);

	let count = store.list::<Ticket>(None)?;
	assert_eq!(TICKET_TITLES.len() - 1, count.len());

	Ok(())
}

#[test]
fn test_crud_list_all() -> anyhow::Result<()> {
	let store = seed_store()?;

	let count = store.list::<Ticket>(None)?;
	assert_eq!(TICKET_TITLES.len(), count.len());

	let count = store.list::<Project>(None)?;
	assert_eq!(PROJECT_NAMES.len(), count.len());

	Ok(())
}

#[test]
fn test_crud_get_simple() -> anyhow::Result<()> {
	let store = seed_store()?;

	let filter: FilterNode = ("id", 100).into();
	let ticket = store.first::<Ticket>(filter)?.context("Should have found ticket for #100")?;

	assert_eq!(100, ticket.id);
	assert_eq!(TICKET_TITLES[0], ticket.title);

	Ok(())
}

#[test]
fn test_crud_list_filter_byebye() -> anyhow::Result<()> {
	let store = seed_store()?;

	let filter: FilterNode = ("title", "byebye all").into();
	let count = store.list::<Ticket>(filter)?;
	assert_eq!(2, count.len());

	Ok(())
}

#[test]
fn test_crud_list_update_byebye() -> anyhow::Result<()> {
	let store = seed_store()?;

	let filter: FilterNode = ("title", "byebye all").into();
	let count = store.update::<Ticket, _>(filter, |mut t| {
		t.title = "heyhey".to_string();
		t
	})?;
	assert_eq!(2, count);

	// 'byebye all' should be 0
	let count = store.list::<Ticket>(FilterNode::from(("title", "byebye all")))?;
	assert_eq!(0, count.len());

	// 'heyhey' should be 2
	let count = store.list::<Ticket>(FilterNode::from(("title", "heyhey")))?;
	assert_eq!(2, count.len());

	Ok(())
}
