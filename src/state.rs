use std::collections::HashMap;

pub struct State {
    pub(crate) documents: HashMap<String, String>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub(crate) fn add_document(&mut self, name: String, doc: String) {
        self.documents.insert(name, doc);
    }

    pub(crate) fn update_document(&mut self, name: String, doc: String) {
        self.documents.insert(name, doc);
    }

    pub(crate) fn get_document(&self, name: &str) -> Option<&String> {
        self.documents.get(name)
    }
}
