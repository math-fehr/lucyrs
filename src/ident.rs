use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct IdentGenerator {
    base: Rc<RefCell<IdentGeneratorBase>>,
    index: u32,
}

#[derive(Debug, Clone)]
struct IdentGeneratorBase {
    name: String,
    available_index: u32,
}

impl IdentGenerator {
    pub fn new(name: String) -> IdentGenerator {
        let base = Rc::new(RefCell::new(IdentGeneratorBase {
            name,
            available_index: 1,
        }));
        IdentGenerator { base, index: 0 }
    }

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

    pub fn get_ident(&self) -> String {
        gen_ident(self.base.borrow().name.clone(), self.index)
    }
}

pub fn gen_ident(s: String, index: u32) -> String {
    if index == 0 {
        s + "_"
    } else {
        s + "_" + &(index - 1).to_string()
    }
}
