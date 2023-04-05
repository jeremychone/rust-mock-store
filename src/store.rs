use crate::{Error, Result};
use modql::filter::FilterGroups;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_value, to_value, Value};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Store {
	stores: Arc<Mutex<HashMap<TypeId, ObjectStore>>>,
}

impl Store {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self { stores: Arc::default() }
	}
}

// region:    --- Store Public Interface
impl Store {
	/// Insert a new value to the underlying store for this type.
	///
	/// Fails if it cannot call `serde_json::to_value`
	pub fn insert<T: Serialize + 'static>(&self, val: T) -> Result<()> {
		// -- Get or create the object store.
		let mut stores = self.stores.lock().unwrap();
		let tid = TypeId::of::<T>();
		let obj_store = stores.entry(tid).or_insert_with(ObjectStore::default).clone();
		drop(stores);

		// -- Convert to json value.
		let val = to_value(val)?;

		// -- Insert to object store.
		obj_store.insert(val);

		Ok(())
	}

	/// List all of the values for a given Type application the Filter
	pub fn list<T>(&self, filter_groups: impl Into<Option<FilterGroups>>) -> Result<Vec<T>>
	where
		T: DeserializeOwned + 'static,
	{
		// -- Get the object store.
		let stores = self.stores.lock().unwrap();
		let tid = TypeId::of::<T>();
		let obj_store = stores.get(&tid).cloned();
		drop(stores);

		// -- If no store, return empty array.
		let Some(obj_store) = obj_store else {
			return Ok(Vec::new())
		};

		// -- Perform the list and filter if object store.
		let filter_groups = filter_groups.into();
		let objects: std::result::Result<Vec<T>, serde_json::Error> =
			obj_store.list(&filter_groups).into_iter().map(|v| from_value::<T>(v)).collect();

		Ok(objects?)
	}

	pub fn delete<T>(&self, filter_groups: impl Into<FilterGroups>) -> Result<u64>
	where
		T: DeserializeOwned + 'static,
	{
		// -- Get the object store.
		let stores = self.stores.lock().unwrap();
		let tid = TypeId::of::<T>();
		let obj_store = stores.get(&tid).cloned();
		drop(stores);

		// -- If no store, return error.
		let Some(obj_store) = obj_store else {
			return Err(Error::FailToDeleteNoStoreForType)
		};

		// -- Perform the delete.
		let filter_groups = filter_groups.into();
		let count = obj_store.delete(&filter_groups)?;

		Ok(count)
	}

	/// Update zero or more item in a type store, for a given filter_groups with the modifier function.
	///
	/// Some limitations:
	/// - It will few clones to keep the API ergonomic convenient. (might try to use `Deserialize<'_>`)
	/// - For now, it will stop at first error. Eventually, we might capture the error an continue.
	/// - Obviously, do not have commit/rollback capability. This is just a mock-store.
	pub fn update<T, F>(&self, filter_groups: impl Into<FilterGroups>, modifier: F) -> Result<u64>
	where
		T: Serialize + DeserializeOwned + 'static,
		F: Fn(T) -> T,
	{
		// -- Get the object store.
		let stores = self.stores.lock().unwrap();
		let tid = TypeId::of::<T>();

		// -- If no store, return error.
		let Some(obj_store) = stores.get(&tid) else {
			return Err(Error::FailToUpdateNoStoreForType)
		};

		// -- Perform the update.
		let filter_groups = filter_groups.into();
		let count = obj_store.update_raw(&filter_groups, |v| {
			let obj = from_value::<T>(v.clone())?;
			let obj = modifier(obj);
			let new_v = to_value(obj)?;
			*v = new_v;
			Ok(())
		})?;

		Ok(count)
	}
}
// endregion: --- Store Public Interface

// region:    --- ObjectStore
#[derive(Clone, Default)]
struct ObjectStore {
	store: Arc<Mutex<Vec<Value>>>,
}

impl ObjectStore {
	fn insert(&self, val: Value) {
		let mut store = self.store.lock().unwrap();
		store.push(val);
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
