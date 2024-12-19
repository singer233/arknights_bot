use uuid::Uuid;

#[derive(Debug)]
pub struct ObjectStorage<T> {
    storage: std::collections::HashMap<Uuid, T>,
}
impl <T> ObjectStorage<T> {
    pub fn new() -> Self {
        ObjectStorage {
            storage: std::collections::HashMap::new(),
        }
    }
    #[must_use]
    pub fn insert(&mut self, value: T) -> Uuid {
        let uuid = self.gen_uuid();
        self.storage.insert(uuid, value);
        uuid
    }

    #[inline]
    fn gen_uuid(&self)->Uuid{
        let mut new_uuid = Uuid::new_v4();
        while self.storage.contains_key(&new_uuid) {
            new_uuid = Uuid::new_v4();
        }
        new_uuid
    }
    pub fn get(&self, uuid: Uuid) -> Option<&T> {
        self.storage.get(&uuid)
    }
    pub fn get_mut(&mut self, uuid: &Uuid) -> Option<&mut T> {
        self.storage.get_mut(uuid)
    }
    pub fn remove(&mut self, uuid: &Uuid) -> Option<T> {
        self.storage.remove(uuid)
    }
}