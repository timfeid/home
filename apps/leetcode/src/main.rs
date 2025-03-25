use std::cmp::Ordering;

mod morse;
mod name_chain;
mod sleep;

struct MyHashMap {
    map: Vec<(i32, i32)>,
}

/**
 * `&self` means the method takes an immutable reference.
 * If you need a mutable reference, change it to `&mut self` instead.
 */
impl MyHashMap {
    fn new() -> Self {
        Self { map: vec![] }
    }

    fn put(&mut self, key: i32, value: i32) {
        for (map_key, map_value) in self.map.iter_mut() {
            if *map_key == key {
                *map_value = value;
                return;
            }
        }

        self.map.push((key, value));
    }

    fn get(&self, key: i32) -> i32 {
        for (map_key, map_value) in self.map.iter() {
            if *map_key == key {
                return *map_value;
            }
        }

        return -1;
    }

    fn remove(&mut self, key: i32) {
        for (i, (map_key, _)) in self.map.iter().enumerate() {
            if *map_key == key {
                self.map.remove(i);
                return;
            }
        }
    }
}

fn find_meetings(start_times: Vec<i32>, end_times: Vec<i32>) -> Vec<usize> {
    let mut meetings: Vec<(usize, &i32, i32)> = start_times
        .iter()
        .enumerate()
        .map(|(ind, start_time)| {
            let end_time = end_times[ind];
            (ind, start_time, end_time)
        })
        .collect();
    meetings.sort_by_key(|&(_, _, x)| x);

    let mut last_finish_time = 0;
    let mut results = vec![];

    for (index, start, end) in meetings {
        if *start >= last_finish_time {
            results.push(index + 1);
            last_finish_time = end;
        }
    }

    results
}

fn main() {
    // let mut obj = MyHashMap::new();
    // obj.put(1, 1);
    // println!("added 1,1 map: {:?}", obj.map);
    // obj.put(2, 2);
    // println!("added 2,2 map: {:?}", obj.map);
    // println!("get 1: {}", obj.get(1));
    // println!("get 3: {}", obj.get(3));
    // obj.put(2, 1);
    // println!("added 2,1 map: {:?}", obj.map);
    // println!("get 2: {}", obj.get(2));
    // obj.remove(2);
    // println!("remove 2: {:?}", obj.map);
    // println!("get 2: {}", obj.get(2));
    // println!(
    //     "output: {:?}",
    //     find_meetings(vec![1, 3, 0, 5, 8, 5], vec![2, 4, 6, 7, 9, 9])
    // );
    println!(
        "output: {:?}",
        find_meetings(
            vec![75250, 50074, 43659, 8931, 11273, 27545, 50879, 77924],
            vec![112960, 114515, 81825, 93424, 54316, 35533, 73383, 160252]
        )
    );
}
