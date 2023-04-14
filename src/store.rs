use crate::{Error, Result};
use modql::filter::FilterGroups;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_value, to_value, Value};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct Store {
	stores: Arc<Mutex<HashMap<TypeId, ObjectStore>>>,
}

impl Store {
	/// Same as Store::default()
	pub fn new() -> Self {
		Self::default()
	}
}

// region:    --- Store Public Interface
impl Store {
	/// Each object store have a u64 sequential id generator which will increment on each call.
	/// Notes:
	///   - The first `seq_next()` call will return `1` since is initialize at 0.
	///   - Also, if the object store does it exists yet, it will create it.
	pub fn seq_next<T: Serialize + 'static>(&self) -> u64 {
		self.get_or_create_obj_store::<T>().next_seq()
	}

	/// Insert a new value to the underlying store for this type.
	///
	/// Fails if it cannot call `serde_json::to_value`
	pub fn insert<T: Serialize + 'static>(&self, val: T) -> Result<()> {
		// -- Get or create the object store.
		let obj_store = self.get_or_create_obj_store::<T>();

		// -- Convert to json value.
		let val = to_value(val)?;

		// -- Insert to object store.
		obj_store.insert(val);

		Ok(())
	}

	pub fn first<T>(&self, filter: impl Into<Option<FilterGroups>>) -> Result<Option<T>>
	where
		T: DeserializeOwned + 'static,
	{
		// -- Get the object store.
		let obj_store = self.get_obj_store::<T>();

		// -- If no store, return empty array.
		let Some(obj_store) = obj_store else {
			return Ok(None);
		};

		// -- Perform the list and filter if object store.
		let filter_groups = filter.into();

		let res = obj_store.first(&filter_groups).map(|v| from_value::<T>(v)).transpose();

		Ok(res?)
	}

	/// List all of the values for a given Type application the Filter
	pub fn list<T>(&self, filter: impl Into<Option<FilterGroups>>) -> Result<Vec<T>>
	where
		T: DeserializeOwned + 'static,
	{
		// -- Get the object store.
		let obj_store = self.get_obj_store::<T>();

		// -- If no store, return empty array.
		let Some(obj_store) = obj_store else {
			return Ok(Vec::new())
		};

		// -- Perform the list and filter if object store.
		let filter_groups = filter.into();
		let objects: std::result::Result<Vec<T>, serde_json::Error> =
			obj_store.list(&filter_groups).into_iter().map(|v| from_value::<T>(v)).collect();

		Ok(objects?)
	}

	pub fn delete<T>(&self, filter: impl Into<FilterGroups>) -> Result<u64>
	where
		T: DeserializeOwned + 'static,
	{
		// -- Get the object store.
		let obj_store = self.get_obj_store::<T>().ok_or(Error::FailToDeleteNoStoreForType)?;

		// -- Perform the delete.
		let filter_groups = filter.into();
		let count = obj_store.delete(&filter_groups)?;

		Ok(count)
	}

	/// Update zero or more item in a type store, for a given filter_groups with the modifier function.
	///
	/// Some limitations:
	/// - It will few clones to keep the API ergonomic convenient. (might try to use `Deserialize<'_>`)
	/// - For now, it will stop at first error. Eventually, we might capture the error an continue.
	/// - Obviously, do not have commit/rollback capability. This is just a mock-store.
	pub fn update<T, F>(&self, filter: impl Into<FilterGroups>, modifier: F) -> Result<u64>
	where
		T: Serialize + DeserializeOwned + 'static,
		F: Fn(T) -> T,
	{
		// -- Get the object store.
		let obj_store = self.get_obj_store::<T>().ok_or(Error::FailToUpdateNoStoreForType)?;

		// -- Perform the update.
		let filter_groups = filter.into();
		let count = obj_store.update_raw(&filter_groups, |v| {
			let obj = from_value::<T>(v.clone())?;
			let obj = modifier(obj);
			let new_v = to_value(obj)?;
			*v = new_v;
			Ok(())
		})?;

		Ok(count)
	}

	// region:    --- Privates
	fn get_obj_store<T: 'static>(&self) -> Option<ObjectStore> {
		let stores = self.stores.lock().unwrap();
		let tid = TypeId::of::<T>();
		stores.get(&tid).cloned()
	}

	fn get_or_create_obj_store<T: Serialize + 'static>(&self) -> ObjectStore {
		let mut stores = self.stores.lock().unwrap();
		let tid = TypeId::of::<T>();
		stores.entry(tid).or_insert_with(ObjectStore::default).clone()
	}
	// endregion: --- Privates
}
// endregion: --- Store Public Interface

// region:    --- ObjectStore
#[derive(Clone, Default)]
struct ObjectStore {
	store: Arc<Mutex<Vec<Value>>>,
	seq: Arc<AtomicU64>,
}

impl ObjectStore {
	fn next_seq(&self) -> u64 {
		self.seq.fetch_add(1, Ordering::Relaxed) + 1
	}

	fn insert(&self, val: Value) {
		let mut store = self.store.lock().unwrap();
		store.push(val);
	}

	fn first(&self, filter_groups: &Option<FilterGroups>) -> Option<Value> {
		let store = self.store.lock().unwrap();

		let filter_groups = filter_groups.as_ref();

		store
			.iter()
			.find(|v| {
				filter_groups
					.map(|filter_groups| filter_groups.is_match_json(v))
					.unwrap_or(true)
			})
			.cloned()
	}

	fn list(&self, filter_groups: &Option<FilterGroups>) -> Vec<Value> {
		let store = self.store.lock().unwrap();

		let filter_groups = filter_groups.as_ref();
		store
			.iter()
			.filter(|v| {
				filter_groups
					.map(|filter_groups| filter_groups.is_match_json(v))
					.unwrap_or(true)
			})
			.cloned()
			.collect()
	}

	fn delete(&self, filter_groups: &FilterGroups) -> Result<u64> {
		let mut store = self.store.lock().unwrap();

		let mut count: u64 = 0;

		// TODO: Need to optimize.
		//   - Option 1: Get indexes and do swap_remove (remaining order not preserved)
		//   - Option 2: Can use `drain_filter` when stable.
		store.retain(|v| {
			let retain = !filter_groups.is_match_json(v);
			if !retain {
				count += 1;
			}
			retain
		});

		Ok(count)
	}

	fn update_raw<F>(&self, filter_groups: &FilterGroups, raw_modifier: F) -> Result<u64>
	where
		F: Fn(&mut Value) -> Result<()>,
	{
		let mut store = self.store.lock().unwrap();

		let mut count: u64 = 0;

		for value in store.iter_mut() {
			if filter_groups.is_match_json(value) {
				raw_modifier(value)?;
				count += 1;
			}
		}

		Ok(count)
	}
}
// endregion: --- ObjectStore
