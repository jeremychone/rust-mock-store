use crate::common::{seed_store, Project, Ticket, PROJECT_NAMES, TICKET_TITLES};
use mock_store::Store;
use modql::filter::FilterNode;

mod common;

#[test]
fn test_crust_create_one() -> anyhow::Result<()> {
	let store = Store::new();

	store.insert(Ticket::new(1, "hello one"))?;

	let tickets = store.list::<Ticket>(None)?;

	assert_eq!("hello one", tickets[0].title);

	Ok(())
}

#[test]
fn test_crust_delete_one() -> anyhow::Result<()> {
	let store = seed_store()?;

	let count = store.delete::<Ticket>(FilterNode::from(("id", 101)))?;
	assert_eq!(1, count);

	let count = store.list::<Ticket>(None)?;
	assert_eq!(TICKET_TITLES.len() - 1, count.len());

	Ok(())
}

#[test]
fn test_crust_list_all() -> anyhow::Result<()> {
	let store = seed_store()?;

	let count = store.list::<Ticket>(None)?;
	assert_eq!(TICKET_TITLES.len(), count.len());

	let count = store.list::<Project>(None)?;
	assert_eq!(PROJECT_NAMES.len(), count.len());

	Ok(())
}

#[test]
fn test_crust_list_filter_byebye() -> anyhow::Result<()> {
	let store = seed_store()?;

	let filter: FilterNode = ("title", "byebye all").into();
	let count = store.list::<Ticket>(filter)?;
	assert_eq!(2, count.len());

	Ok(())
}

#[test]
fn test_crust_list_update_byebye() -> anyhow::Result<()> {
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
