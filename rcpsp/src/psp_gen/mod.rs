use rand::Rng;

pub fn _gen_psp() {
    // number of jobs
    let n = rand::thread_rng().gen_range(8..30);
    //jobs
    let _v: Vec<i32> = (0..n).collect();
    // duration of jobs (randomly with 0 for 0th and Nth
    let _d: Vec<i32> = (0..n)
        .map(|x| {
            if x == 0 || x == n {
                0
            } else {
                rand::thread_rng().gen_range(5..25)
            }
        })
        .collect();
    //DAG of the Problem
    let _e: Vec<Vec<bool>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    if i <= j {
                        false
                    }
                    //obere Dreicksmatrix
                    else if i == 0 || i == n - 1 {
                        true
                    }
                    //Anfangs/Endpunkt
                    else {
                        let t: i32 = rand::thread_rng().gen_range(0..3);
                        t == 0
                    } //chance is 1/3 here for edges
                })
                .collect()
        })
        .collect();
    //number of ressources
    let m = rand::thread_rng().gen_range(3..7);
    //maximal resources of system
    let rmax: Vec<i32> = (0..m)
        .map(|_| rand::thread_rng().gen_range(6..14))
        .collect();
    //ressource matrix
    let _r: Vec<Vec<i32>> = (0..n)
        .map(|_| {
            (0..m)
                .map(|j| {
                    //makes lower values where one process wont need more then
                    // half the ressources of one type more likely, so sheduling makes more sense
                    let t1 = rand::thread_rng().gen_range(0..rmax[j] + 1);
                    let t2 = rand::thread_rng().gen_range(0..rmax[j] + 1);
                    t1 * t2 / rmax[j]
                })
                .collect()
        })
        .collect();
}
