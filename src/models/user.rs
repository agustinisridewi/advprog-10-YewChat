use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct User {
    pub username: Rc<RefCell<String>>,
    pub avatar: Rc<RefCell<String>>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            username: Rc::new(RefCell::new(String::new())),
            avatar: Rc::new(RefCell::new(String::from("alex"))), // Default avatar
        }
    }
}