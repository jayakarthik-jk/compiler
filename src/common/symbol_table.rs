use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::common::datatypes::DataType;
pub struct SymbolTable {
    pub variables: HashMap<String, DataType>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            variables: HashMap::new(),
        }
    }

    pub fn sharable() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(SymbolTable::new()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &DataType)> {
        self.variables.iter()
    }

    pub fn print(&self) {
        for (name, value) in self.iter() {
            println!("{}: {}", name, value);
        }
    }
}
