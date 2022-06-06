use super::SourcesLoad;

pub struct TimeResolution {
    number_of_resources: usize,
    makespan_upper_bound: usize,
    remaining_resource_capacity: Vec<Vec<usize>>,
}

impl TimeResolution {
    pub fn new(
        number_of_resources: usize,
        capacity_of_resources: Vec<usize>,
        makespan_upper_bound: usize,
    ) -> Self {
        let mut remaining_resource_capacity = Vec::with_capacity(number_of_resources);

        for resource_id in 0..(number_of_resources) {
            let capacity_of_resource: usize = match capacity_of_resources.get(resource_id) {
                Some(capacity) => *capacity,
                None => 0,
            };

            let mut capacity_vec = Vec::with_capacity(makespan_upper_bound);

            for _ in 0..capacity_vec.capacity() {
                capacity_vec.push(capacity_of_resource);
            }

            remaining_resource_capacity.push(capacity_vec);
        }

        Self {
            number_of_resources,
            makespan_upper_bound,
            remaining_resource_capacity,
        }
    }
}

impl SourcesLoad for TimeResolution {
    fn get_earliest_start_time(
        &self,
        activity_resource_requirements: &[usize],
        earliest_precedence_start_time: usize,
        activity_duration: usize,
    ) -> usize {
        let mut load_time: usize = 0;
        let mut t: usize = earliest_precedence_start_time;

        while t < self.makespan_upper_bound && load_time < activity_duration {
            let mut capacity_available = true;

            for resource_id in 0..self.number_of_resources {
                if !capacity_available {
                    break;
                }

                if let Some(remaining_resource_capacity) =
                    self.remaining_resource_capacity.get(resource_id)
                {
                    if let Some(remaining_resource_capacity) = remaining_resource_capacity.get(t) {
                        if let Some(activity_resource_requirement) =
                            activity_resource_requirements.get(resource_id)
                        {
                            if remaining_resource_capacity < activity_resource_requirement {
                                load_time = 0;
                                capacity_available = false;
                            }
                        }
                    }
                }
            }

            if capacity_available {
                load_time += 1;
            }

            t += 1;
        }

        t - load_time
    }

    fn add_activity(
        &mut self,
        activity_start: usize,
        activity_stop: usize,
        activity_requirements: &[usize],
    ) {
        for resource_id in 0..self.number_of_resources {
            for t in activity_start..activity_stop {
                if let Some(remaining_resource_capacity) =
                    self.remaining_resource_capacity.get_mut(resource_id)
                {
                    if let Some(remaining_resource_capacity) =
                        remaining_resource_capacity.get_mut(t)
                    {
                        if let Some(activity_requirement) = activity_requirements.get(resource_id) {
                            *remaining_resource_capacity -= *activity_requirement;
                        }
                    }
                }
            }
        }
    }
}
