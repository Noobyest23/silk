use std::{collections::HashMap, str, usize};

#[derive(Clone)]
pub struct Scope {
    pub variables: HashMap<String, usize>,
    parent: Option<Box<Scope>>
}

impl Scope {
    
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: Option::None,
        }
    }

    pub fn retrieve(&mut self, id: &str) -> Option<usize> {
        let v = self.variables.get(id);

        if v.is_none() {
            if let Some(mut p) = self.parent.clone() {
                return p.retrieve(id);
            }
        }
        v.copied()
    }

    pub fn set_global(&mut self, id: &str, ptr: usize) {
        if let Some(mut p) = self.parent.clone() {
            p.set_global(id, ptr);
        }
        else {
            self.variables.insert(id.to_string(), ptr);
        }
    }

    pub fn child(&self) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Option::Some(Box::new(self.clone()))
        }
    }

    pub fn pop(&mut self) -> Self {
        if let Some(p) = &self.parent {
            return *p.clone();
        }
        panic!("Cannot pop out of scope with no parent")
    }

}