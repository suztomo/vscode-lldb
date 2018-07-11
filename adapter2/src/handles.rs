use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::rc::Rc;

pub type Handle = NonZeroU32;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct VPath(Rc<(String, Option<VPath>)>);

impl VPath {
    pub fn new(key: &str) -> VPath {
        VPath(Rc::new((key.to_owned(), None)))
    }
    pub fn extend(&self, key: &str) -> VPath {
        VPath(Rc::new((key.to_owned(), Some(self.clone()))))
    }
}

pub struct HandleTree<Value> {
    obj_by_handle: HashMap<Handle, (Value, VPath)>,
    handle_by_vpath: HashMap<VPath, Handle>,
    prev_handle_by_vpath: HashMap<VPath, Handle>,
    next_handle_value: u32,
}

impl<Value> HandleTree<Value> {
    pub fn new() -> Self {
        HandleTree {
            obj_by_handle: HashMap::new(),
            handle_by_vpath: HashMap::new(),
            prev_handle_by_vpath: HashMap::new(),
            next_handle_value: 1000,
        }
    }

    pub fn from_prev(mut old: HandleTree<Value>) -> Self {
        old.obj_by_handle.clear();
        old.prev_handle_by_vpath.clear();

        HandleTree {
            obj_by_handle: old.obj_by_handle,
            handle_by_vpath: old.prev_handle_by_vpath,
            prev_handle_by_vpath: old.handle_by_vpath,
            next_handle_value: old.next_handle_value,
        }
    }

    pub fn create(&mut self, parent_handle: Option<Handle>, key: &str, value: Value) -> Handle {
        let new_vpath = match parent_handle {
            Some(parent_handle) => {
                let (_, parent_vpath) = self.obj_by_handle.get(&parent_handle).unwrap();
                parent_vpath.extend(key)
            }
            None => VPath::new(key),
        };

        let new_handle = match self.prev_handle_by_vpath.get(&new_vpath) {
            Some(handle) => handle.clone(),
            None => {
                self.next_handle_value += 1;
                Handle::new(self.next_handle_value).unwrap()
            }
        };

        self.handle_by_vpath.insert(new_vpath.clone(), new_handle);
        self.obj_by_handle.insert(new_handle, (value, new_vpath));
        new_handle
    }

    pub fn get(&self, handle: Handle) -> Option<&Value> {
        self.obj_by_handle.get(&handle).map(|t| &t.0)
    }

    pub fn get_with_vpath(&self, handle: Handle) -> Option<&(Value, VPath)> {
        self.obj_by_handle.get(&handle)
    }
}

#[test]
fn test1() {
    let mut handles = HandleTree::new();
    let a1 = handles.create(None, "1", 0xa1);
    let a2 = handles.create(None, "2", 0xa2);
    let a11 = handles.create(Some(a1), "1.1", 0xa11);
    let a12 = handles.create(Some(a1), "1.2", 0xa12);
    let a121 = handles.create(Some(a12), "1.2.1", 0xa121);
    let a21 = handles.create(Some(a2), "2.1", 0xa21);

    assert!(handles.get(a1).unwrap() == &0xa1);
    assert!(handles.get(a12).unwrap() == &0xa12);
    assert!(handles.get(a121).unwrap() == &0xa121);

    let mut handles2 = HandleTree::from_prev(handles);
    let b1 = handles2.create(None, "1", 0xb1);
    let b3 = handles2.create(None, "3", 0xb3);
    let b11 = handles2.create(Some(b1), "1.1", 0xb11);
    let b12 = handles2.create(Some(b1), "1.2", 0xb12);
    let b13 = handles2.create(Some(b1), "1.3", 0xb13);
    let b121 = handles2.create(Some(b12), "1.2.1", 0xb121);
    let b122 = handles2.create(Some(b12), "1.2.2", 0xb122);

    assert!(handles2.get(a2) == None);
    assert!(handles2.get(a21) == None);

    assert!(b1 == a1);
    assert!(b11 == a11);
    assert!(b12 == a12);
    assert!(b121 == a121);

    assert!(handles2.get(b1).unwrap() == &0xb1);
    assert!(handles2.get(b122).unwrap() == &0xb122);
}

#[test]
#[should_panic]
fn test2() {
    let mut handles = HandleTree::new();
    let h1 = handles.create(None, "12345", 12345);
    let h2 = handles.create(Some(Handle::new(h1.get() + 1).unwrap()), "12345", 12345);
}