use std::collections::HashSet;
use std::collections::HashMap;
// use hashbrown::HashMap;
use super::ObjIdTraits;
use rand::seq::IteratorRandom;


pub const MAX_EXPIRING_VEC_SIZE : usize = 10000000;
#[derive(Clone)]
pub struct LeaseCache <ObjId:ObjIdTraits> {
    pub(crate) expiring_vec : Vec<HashSet<ObjId>>,
    pub(crate) curr_expiring_index : usize,
    //map from ObjId to index in expiring_vec 
    pub(crate) content_map : HashMap<ObjId, usize>,
}
impl <ObjId:ObjIdTraits> LeaseCache <ObjId> {
    pub fn new() -> Self {
        LeaseCache {
            expiring_vec : vec![HashSet::new(); MAX_EXPIRING_VEC_SIZE],
            curr_expiring_index : 0,
            content_map : HashMap::new(),
        }
    }

    pub fn insert(&mut self, obj_id: ObjId, lease: usize) {
        let absolute_index = (self.curr_expiring_index + lease) % MAX_EXPIRING_VEC_SIZE;
        self.expiring_vec[absolute_index].insert(obj_id.clone());
        self.content_map.insert(obj_id, absolute_index);
    }

    pub fn update(&mut self, obj_id: &ObjId, lease: usize) {
        let old_index = self.content_map.get(obj_id);
        match old_index {
            None => self.insert(obj_id.clone(), lease),
            Some(old_index) => {
                self.expiring_vec[*old_index].remove(obj_id);
                self.insert(obj_id.clone(), lease)
            }
        }
    }

    pub fn contains(&self, obj_id: &ObjId) -> bool {
        self.content_map.contains_key(obj_id)
    }

    pub fn get_time_till_eviction(&self, obj_id: &ObjId) -> usize {
        let index = self.content_map.get(obj_id).unwrap();
        let curr_index = self.curr_expiring_index;
        if *index > curr_index {
            return *index - curr_index;
        }
        return MAX_EXPIRING_VEC_SIZE - curr_index + *index;
    }

    pub fn remove_from_cache(&mut self, obj_id: &ObjId) {
        let index = self.content_map.get(obj_id).unwrap();
        self.expiring_vec[*index].remove(obj_id);
        self.content_map.remove(obj_id);
    }

    pub fn dump_expiring(&mut self) -> HashSet<ObjId> {
        let mut expiring = self.expiring_vec[self.curr_expiring_index].clone();
        let expiring_copy = expiring.clone();
        expiring.clear();
        self.curr_expiring_index = (self.curr_expiring_index + 1) % MAX_EXPIRING_VEC_SIZE;
        return expiring_copy
    }

    fn remove_random_element<K, V>(map: &mut HashMap<K, V>) -> Option<(K, V)>
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
    {
        if let Some((key, val)) = map.clone().iter().choose(&mut rand::thread_rng()) {
            map.remove(key);
            return Some((key.clone(), val.clone()));
        }
    None
    }

    pub fn force_evict(&mut self) -> ObjId{
        // println!("content map before {:?}", self.content_map);

        let (obj_id, absolute_index) = LeaseCache::<ObjId>::remove_random_element(&mut self.content_map).unwrap();

        self.expiring_vec[absolute_index].remove(&obj_id.clone()).then_some(()).unwrap();
        obj_id
    }
}

#[cfg(test)]

mod test {
    use super::*;
    #[test]
    fn test_lease_cache_new() {
        let lease_cache = LeaseCache::<usize>::new();
        assert_eq!(lease_cache.expiring_vec.len(), crate::simulator::lease_cache::MAX_EXPIRING_VEC_SIZE);
        assert_eq!(lease_cache.curr_expiring_index, 0);
        assert_eq!(lease_cache.content_map.len(), 0);
    }

    #[test]
    fn test_lease_cache_insert() {
        let mut lease_cache = LeaseCache::new();
        lease_cache.insert(1, 1);
        lease_cache.insert(2, 2);
        lease_cache.insert(3, 3);
        assert!(lease_cache.content_map.contains_key(&1));
        assert!(lease_cache.content_map.contains_key(&2));
        assert!(lease_cache.content_map.contains_key(&3));
        let mut abs_index = lease_cache.content_map.get(&1).unwrap();
        assert_eq!(lease_cache.expiring_vec[*abs_index].contains(&1), true);
        abs_index = lease_cache.content_map.get(&2).unwrap();
        assert_eq!(lease_cache.expiring_vec[*abs_index].contains(&2), true);
        abs_index = lease_cache.content_map.get(&3).unwrap();
        assert_eq!(lease_cache.expiring_vec[*abs_index].contains(&3), true);
    }

    // #[test]
    // fn test_lease_cache_update() {
    //     let mut lease_cache = LeaseCache::new();
    //     lease_cache.update(1, 1);
    //     let abs_index = lease_cache.content_map.get(&1).unwrap();
    //     assert!(lease_cache.expiring_vec[*abs_index].contains(&1));
    //     lease_cache.update(1, 4);
    //     assert!(!lease_cache.expiring_vec[*abs_index].contains(&1));
    //     let abs_index = lease_cache.content_map.get(&1).unwrap();
    //     assert!(lease_cache.expiring_vec[*abs_index].contains(&1));
        
    // }

    #[test]
    fn test_lease_cache_update() {
        let mut lease_cache = LeaseCache::new();
        // Update the lease cache with obj_id 1 and index 1
        lease_cache.update(&1, 1);
        // Get the absolute index of obj_id 1 and release the immutable borrow
        let abs_index: usize = *lease_cache.content_map.get(&1).unwrap();
        assert!(lease_cache.expiring_vec[abs_index].contains(&1));
        // Update the lease cache with obj_id 1 and new index 4
        lease_cache.update(&1, 4);
        // Get the old absolute index and assert it no longer contains obj_id 1
        let abs_index_old = abs_index; // Reuse the old index
        assert!(!lease_cache.expiring_vec[abs_index_old].contains(&1));
        // Get the new absolute index and assert it contains obj_id 1
        let abs_index_new = *lease_cache.content_map.get(&1).unwrap();
        assert!(lease_cache.expiring_vec[abs_index_new].contains(&1));
    }

    #[test]
    fn test_lease_cache_dump_expiring() {
        let mut lease_cache = LeaseCache::new();
        //test to make sure expiring objects are dumped correctly,
        //this means that each time we dump we see the objects that the correct
        //objects are expiring and the expiring index is incremented by one
        lease_cache.insert(1, 1);
        lease_cache.insert(2, 2);
        lease_cache.insert(3, 3);
        let mut expiring = lease_cache.dump_expiring();
        assert_eq!(expiring, HashSet::new());
        expiring = lease_cache.dump_expiring();
        let mut expected = HashSet::new();
        expected.insert(1);
        assert_eq!(expiring, expected); 
        expiring = lease_cache.dump_expiring();
        expected.insert(2);
        expected.remove(&1);
        assert_eq!(expiring, expected);
        expiring = lease_cache.dump_expiring();
        expected.insert(3);
        expected.remove(&2);
        assert_eq!(expiring, expected);
        //TODO: test that expiring index is incremented correctly at the boundry
    }


    #[test]
    fn test_lease_cache_force_evict() {
        let epsilon = 0.1;
        let num_iters = 100;
        //we want to test that each object in the cache has an equal chance of being evicted
        let mut num_obj1_evicted = 0;
        let mut num_obj2_evicted = 0;
        let mut num_obj3_evicted = 0;
        for i in 0..num_iters {
            let mut lease_cache = LeaseCache::new();
            lease_cache.insert(1, 100000);
            lease_cache.insert(2, 100000);
            lease_cache.insert(3, 9);
            let evicted_obj = lease_cache.force_evict();
            match evicted_obj {
                1 => num_obj1_evicted += 1,
                2 => num_obj2_evicted += 1,
                3 => num_obj3_evicted += 1,
                _ => panic!("Invalid object evicted")
            }
            // if i % 10 == 1 {
            //     println!("{} ", i)
            // }
        }
        //check that each object was evicted is within a small epsilon
        let check_obj1 = ((num_obj1_evicted as f64 / num_iters as f64) - (1.0/3.0)).abs() < epsilon;
        let check_obj2 = ((num_obj2_evicted as f64 / num_iters as f64) - (1.0/3.0)).abs() < epsilon;
        let check_obj3 = ((num_obj3_evicted as f64 / num_iters as f64) - (1.0/3.0)).abs() < epsilon;
        println!("eviction count: {} {} {}", num_obj1_evicted, num_obj2_evicted, num_obj3_evicted);
        println!("eviction ratio: {} {} {}", num_obj1_evicted as f64 /num_iters as f64, num_obj2_evicted as f64 / num_iters as f64, num_obj3_evicted as f64 / num_iters as f64);
        assert!(check_obj1 && check_obj2 && check_obj3);
    }

    #[test]
    fn test_lease_cache_force_evict_string() {
        let epsilon = 0.1;
        let num_iters = 100;
        //we want to test that each object in the cache has an equal chance of being evicted
        let mut num_obj1_evicted = 0;
        let mut num_obj2_evicted = 0;
        let mut num_obj3_evicted = 0;
        let obj_1 = "x1";
        let obj_2 = "x2";
        let obj_3 = "x3";
        for i in 0..num_iters {
            let mut lease_cache = LeaseCache::new();
            lease_cache.insert(obj_1.to_string(), 100000);
            lease_cache.insert(obj_2.to_string(), 100000);
            lease_cache.insert(obj_3.to_string(), 9);
            let evicted_obj = lease_cache.force_evict();
            // println!("evicted: {evicted_obj}");
            match evicted_obj.as_str() {
                o if o == obj_1 => num_obj1_evicted += 1,
                o if o == obj_2 => num_obj2_evicted += 1,
                o if o == obj_3 => num_obj3_evicted += 1,
                _ => panic!("Invalid object evicted")
            }
            // if i % 10 == 1 {
            //     println!("{} ", i)
            // }
        }
        //check that each object was evicted is within a small epsilon
        let check_obj1 = ((num_obj1_evicted as f64 / num_iters as f64) - (1.0/3.0)).abs() < epsilon;
        let check_obj2 = ((num_obj2_evicted as f64 / num_iters as f64) - (1.0/3.0)).abs() < epsilon;
        let check_obj3 = ((num_obj3_evicted as f64 / num_iters as f64) - (1.0/3.0)).abs() < epsilon;
        println!("eviction count: {} {} {}", num_obj1_evicted, num_obj2_evicted, num_obj3_evicted);
        println!("eviction ratio: {} {} {}", num_obj1_evicted as f64 /num_iters as f64, num_obj2_evicted as f64 / num_iters as f64, num_obj3_evicted as f64 / num_iters as f64);
        assert!(check_obj1 && check_obj2 && check_obj3);
    }
}