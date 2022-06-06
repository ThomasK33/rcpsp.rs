use super::SourcesLoad;

pub struct CapacityResolution {
    number_of_resources: usize,
    capacity_of_resources: Vec<usize>,
    resources_load: Vec<Vec<usize>>,
    start_values: Vec<usize>,
}

impl CapacityResolution {
    pub fn new(number_of_resources: usize, capacity_of_resources: Vec<usize>) -> Self {
        let mut max_capacity = 0;

        let mut resources_load = Vec::with_capacity(number_of_resources);

        for resource_id in 0..(number_of_resources) {
            let capacity: usize = match capacity_of_resources.get(resource_id) {
                Some(capacity) => (*capacity) as usize,
                None => 0,
            };

            resources_load.push(Vec::with_capacity(capacity));

            max_capacity = max_capacity.max(capacity);
        }

        Self {
            number_of_resources,
            capacity_of_resources,
            resources_load,
            start_values: vec![0; max_capacity],
        }
    }
}

impl SourcesLoad for CapacityResolution {
    fn get_earliest_start_time(
        &self,
        activity_resource_requirements: &[usize],
        _: usize,
        _: usize,
    ) -> usize {
        let mut best_start = 0;

        for resource_id in 0..self.number_of_resources {
            if let Some(&activity_requirement) = activity_resource_requirements.get(resource_id) {
                if activity_requirement > 0 {
                    if let Some(resource_load) = self.resources_load.get(resource_id) {
                        if let Some(&capacity_of_resource) =
                            self.capacity_of_resources.get(resource_id)
                        {
                            let start = resource_load
                                .get(capacity_of_resource - (activity_requirement as usize));

                            if let Some(&start) = start {
                                best_start = best_start.max(start);
                            }
                        }
                    }
                }
            }
        }

        best_start
    }

    fn add_activity(
        &mut self,
        activity_start: usize,
        activity_stop: usize,
        activity_requirements: &[usize],
    ) {
        let mut required_squares: i32;
        let mut time_diff: i32;
        let mut k: usize;
        let mut c: usize;
        let mut new_start_time: usize;

        for resource_id in 0..self.number_of_resources {
            if let Some(&capacity_of_resource) = self.capacity_of_resources.get(resource_id) {
                if let Some(&resource_requirement) = activity_requirements.get(resource_id) {
                    required_squares =
                        (resource_requirement * (activity_stop - activity_start)) as i32;

                    if required_squares > 0 {
                        c = 0;
                        new_start_time = activity_stop;
                        k = self.resources_load[resource_id]
                            .iter()
                            .take(capacity_of_resource)
                            .collect::<Vec<_>>()
                            .partition_point(|&&pred| pred < activity_stop);

                        while required_squares > 0 && k < capacity_of_resource {
                            if let Some(resource_load) = self.resources_load.get_mut(resource_id) {
                                if let Some(resource_load_k) = resource_load.get_mut(k) {
                                    if *resource_load_k < new_start_time {
                                        if c >= resource_requirement {
                                            if let Some(&start_time) =
                                                self.start_values.get(c - resource_requirement)
                                            {
                                                new_start_time = start_time;
                                            }
                                        }

                                        time_diff = (new_start_time
                                            - (*resource_load_k).max(activity_start))
                                            as i32;

                                        if (required_squares - time_diff) > 0 {
                                            required_squares -= time_diff;
                                            if let Some(start_value) =
                                                self.start_values.get_mut(c + 1)
                                            {
                                                *start_value = *resource_load_k;
                                            }
                                            *resource_load_k = new_start_time;
                                        } else {
                                            *resource_load_k = new_start_time - time_diff as usize
                                                + required_squares as usize;
                                        }
                                    }
                                }
                            }

                            k += 1;
                        }
                    }
                }
            }
        }
    }
}
