use rand::seq::SliceRandom;
use rand::thread_rng;

use super::TabuList;

#[derive(Debug, Clone)]
pub struct ListRecord {
    /// Index, activity identification or something else. Maximal value is total number of activities-1.
    i: i32,
    /// Index, activity identification or something else. Maximal value is total number of activities-1.
    j: i32,
}

#[derive(Clone)]
pub struct SimpleTabuList {
    /// Current index at tabu list. (circular buffer)
    cur_idx: usize,
    /// Array of tabu list items. It is tabu list.
    tabu: Vec<ListRecord>,
    /// Tabu hash structure. It's two-dimensional array of boolean (size totalNumberOfActivities x totalNumberOfActivities).
    tabu_search: Vec<Vec<bool>>,
    /// Fixed tabu list size.
    tabu_length: usize,
}

impl SimpleTabuList {
    pub fn new(number_of_activities: usize, length: usize) -> Self {
        Self {
            cur_idx: 0,
            tabu_length: length,
            tabu: vec![ListRecord { i: -1, j: -1 }; length],
            tabu_search: vec![vec![false; number_of_activities]; number_of_activities],
        }
    }
}

impl TabuList for SimpleTabuList {
    fn is_possible_move(&self, i: usize, j: usize) -> bool {
        if let Some(value) = self.tabu_search.get(i).and_then(|tsv| tsv.get(j)) {
            if !*value {
                return true;
            }
        }
        false
    }

    fn add_turn_to_tabu_list(&mut self, i: usize, j: usize) {
        if let Some(tabu) = self.tabu.get_mut(self.cur_idx) {
            if tabu.i != -1 && tabu.j != -1 {
                if let Some(ts) = self
                    .tabu_search
                    .get_mut(tabu.i as usize)
                    .and_then(|tsv| tsv.get_mut(tabu.j as usize))
                {
                    *ts = false;
                }
            }

            tabu.i = i as i32;
            tabu.j = j as i32;

            if let Some(ts) = self.tabu_search.get_mut(i).and_then(|tsv| tsv.get_mut(j)) {
                *ts = true;
            }
        }

        self.cur_idx = (self.cur_idx + 1) % self.tabu_length;
    }

    fn best_solution_found(&self) {}

    fn go_to_next_iter(&self) -> usize {
        0
    }

    fn prune(&mut self) {
        let mut idx_valid_moves: Vec<u32> = vec![];
        let mut count_valid_moves_in_tl: u32 = 0;
        for i in 0..self.tabu_length {
            if let Some(tabu) = self.tabu.get(i) {
                if tabu.i != -1 && tabu.j != -1 {
                    idx_valid_moves.push(i as u32);
                    count_valid_moves_in_tl += 1;
                }
            }
        }

        idx_valid_moves.shuffle(&mut thread_rng());

        let count_moves_to_remove = (0.3 * (count_valid_moves_in_tl as f32)) as u32;

        for m in 0..count_moves_to_remove {
            let move_idx: u32 = idx_valid_moves[m as usize];

            if let Some(tabu) = self.tabu.get_mut(move_idx as usize) {
                if let Some(ts) = self
                    .tabu_search
                    .get_mut(tabu.i as usize)
                    .and_then(|tsv| tsv.get_mut(tabu.j as usize))
                {
                    *ts = false;
                }

                tabu.i = -1;
                tabu.j = -1;
            }
        }
    }
}
