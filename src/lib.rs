
extern crate time;
extern crate url;

use std::collections::HashMap;
use time::Timespec;
use url::Url;

#[derive(Clone, Debug)]
pub struct PlaceAction {
    pub url: Url,
    pub when: Timespec,
    pub event: String,
    // TODO: Think about more complex types -- Or do we want callers just
    // storing JSON here? Can we use std::any::Any and hope to serialize it?
    pub data: String
}

impl PlaceAction {
    pub fn new(url: Url, event: &str, data: &str) -> PlaceAction {
        PlaceAction {
            url: url,
            when: time::get_time(),
            event: String::from(event),
            data: String::from(data),
        }
    }
}

pub trait Store {
    fn record_place_action(&mut self, action: PlaceAction);
    fn query_place_actions<'a>(&'a self, url: &Url) -> &'a [PlaceAction];
}

#[derive(Clone, Debug)]
pub struct MemoryStore {
    data: HashMap<Url, Vec<PlaceAction>>,
    // This allows us to avoid needing to put Option<&'a [PlaceAction]>
    // as the return type for query_place_actions (where None would be used for
    // "no actions"). Instead, we just return a reference to a private sentinel
    // vector, which we ensure is always empty.
    sentinel: Vec<PlaceAction>,
}

impl MemoryStore {
    pub fn new() -> MemoryStore {
        MemoryStore {
            data: HashMap::new(),
            sentinel: Vec::new(),
        }
    }
}

impl Store for MemoryStore {
    fn record_place_action(&mut self, action: PlaceAction) {
        if let Some(v) = self.data.get_mut(&action.url) {
            v.push(action);
            return;
        }
        // Can't use else since it would be 2 simultaneous mutable borrows
        self.data.insert(action.url.clone(), vec![action]);
    }

    fn query_place_actions<'a>(&'a self, url: &Url) -> &'a [PlaceAction] {
        self.data.get(url).map(|v| &v[..]).unwrap_or(&self.sentinel[..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn memory_store() {
        let url1 = Url::parse("https://example.com/foo/bar?quux").unwrap();
        let url2 = Url::parse("https://example.com/baz").unwrap();
        let mut store = MemoryStore::new();
        let start = time::get_time();

        store.record_place_action(PlaceAction::new(url1.clone(), "visit", ""));
        store.record_place_action(PlaceAction::new(url1.clone(), "frobnicate", "quux"));
        store.record_place_action(PlaceAction::new(url1.clone(), "quank", "1002"));
        store.record_place_action(PlaceAction::new(url2.clone(), "asdf", "4321"));

        let actions = store.query_place_actions(&url1);
        assert_eq!(actions.len(), 3);

        for action in actions.iter() {
            assert_eq!(action.url, url1);
            assert!(action.when >= start);
        }

        let action = actions.iter().find(|&x| x.event == "visit").unwrap();
        assert_eq!(action.data, "");

        let action = actions.iter().find(|&x| x.event == "frobnicate").unwrap();
        assert_eq!(action.data, "quux");

        let actions = store.query_place_actions(&url2);
        assert_eq!(actions.len(), 1);

        assert_eq!(actions[0].event, "asdf");
        assert_eq!(actions[0].data, "4321");

        let actions = store.query_place_actions(&Url::parse("https://example.com").unwrap());

        assert_eq!(actions.len(), 0);
    }
}
