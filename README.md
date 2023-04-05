Rust **mock-store** is a simple Rust in-memory mock-store for testing and prototyping (with modql implementation).

Do not use this in production code.

**mock-store** uses [modql](https://github.com/jeremychone/rust-modql) for filtering capability. It's also a great way to experiment with **modql**.

[Example](#example) | [Library Scope](#library-scope) | [Limitations](#limitations)

## Example

See [examples/readme.rs](https://github.com/jeremychone/rust-mock-store/blob/main/examples/readme.rs) for the full working source.

```rs
// -- Store is Send + Sync (backed by Arc/Mutex).
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
// [Ticket { id: 1, title: "Ticket AAA" }]
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
// [Ticket { id: 1, title: "Ticket AAA" }, Ticket { id: 1, title: "TICKET BB - UPDATE" }]
println!("{:<20}: {all_tickets:?}", "all tickets");

// -- Delete is: store.delete::<Ticket>(filter)?;

```

## Library Scope

- This is not intended for production use.
- Primarily for writing tests or proofs of concept without a real data store.
- Prioritizes ergonomics and convenience over performance.

## Current Limitations

- Capability: For now, supports only one level down matches and numbers, booleans, strings.
- Performance: The store is per type, but the object store contains the serialized serde_json Value.
- Performance: Because objects are stored as Value, during an update, objects are deserialized and re-serialized.