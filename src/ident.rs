//! The module contains IdentGenerator, which is used to generate new identifiers

use std::cell::RefCell;
use std::rc::Rc;

/// Struct used to generate new identifiers based an a base one
/// For identifier a, the identifiers generated will be:
/// a_, a_0, a_1, ...
#[derive(Debug, Clone)]
pub struct IdentGenerator {
    base: Rc<RefCell<IdentGeneratorBase>>,
    index: u32,
}

/// Underlying structure of IdentGenerator
#[derive(Debug, Clone)]
struct IdentGeneratorBase {
    name: String,
    available_index: u32,
}

impl IdentGenerator {
    /// Create a new IdentGenerator based on a string
    pub fn new(name: String) -> IdentGenerator {
        let base = Rc::new(RefCell::new(IdentGeneratorBase {
            name,
            available_index: 1,
        }));
        IdentGenerator { base, index: 0 }
    }

    /// Generate a new identifier
    pub fn new_ident(&self) -> IdentGenerator {
        let new_base = self.base.clone();
        let mut base = self.base.borrow_mut();
        let ident = IdentGenerator {
            base: new_base,
            index: base.available_index,
        };
        base.available_index += 1;
        ident
    }

    /// Get the underlying string of the identifier
    pub fn get_ident(&self) -> String {
        gen_ident(self.base.borrow().name.clone(), self.index)
    }
}

/// Generate an new string from a string
pub fn gen_ident(s: String, index: u32) -> String {
    if index == 0 {
        s + "_"
    } else {
        s + "_" + &(index - 1).to_string()
    }
}
