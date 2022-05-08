use shared_types::Activity;

pub(crate) fn generate_activities(activity_count: u8) -> Vec<Activity> {
    let mut activities = vec![];

    // TODO: Randomize this

    activities.push(Activity {
        id: 0,
        duration: 0,
        r0: 0,
        r1: 0,
        successors: vec![],
    });

    for id in 1..(activity_count + 1) {
        activities.push(Activity {
            id,
            duration: 0,
            r0: 0,
            r1: 0,
            successors: vec![1, 1],
        });
    }

    activities.push(Activity {
        id: activity_count + 1,
        duration: 0,
        r0: 0,
        r1: 0,
        successors: vec![],
    });

    activities
}
