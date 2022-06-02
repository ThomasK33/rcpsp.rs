use anyhow::Result;
use rand::Rng;

use crate::Schedule;

pub fn schedule(_schedule: Schedule) -> Result<()> {
    // todo!("Implement scheduler")

    //============ Start Definition of Problem ============
    // number of jobs
    let N= rand::thread_rng().gen_range(8..30);
    //jobs
    let V :Vec<i32> = (0..N).collect();
    // duration of jobs (randomly with 0 for 0th and Nth
    let D :Vec<i32> = (0..N).map(|x| {
            if x==0 || x==N {0}
            else{rand::thread_rng().gen_range(5..25)}
        }).collect();
    //DAG of the Problem
    let E :Vec<Vec<bool>> = (0..N).map(|i| {
            (0..N).map(|j| {
                if i<=j {false} //obere Dreicksmatrix
                else if i==0 || i==N-1 {true} //Anfangs/Endpunkt
                else {rand::thread_rng().gen_range(0..3)==0} //chance is 1/3 here for edges
            }).collect()
        }).collect();
    //number of ressources
    let M = rand::thread_rng().gen_range(3..7);
    //maximal resources of system
    let Rmax :Vec<i32> = (0..M).map(|x| rand::thread_rng().gen_range(6..14)).collect();
    //ressource matrix
    let R :Vec<Vec<i32>> = (0..N).map(|i| {
            (0..M).map(|j| {
                //makes lower values where one process wont need more then
                // half the ressources of one type more likely, so sheduling makes more sense
                let t1=rand::thread_rng().gen_range(0..Rmax[j]+1);
                let t2=rand::thread_rng().gen_range(0..Rmax[j]+1);
                t1*t2/Rmax[j]
            }).collect()
        }).collect();
    //============ End Definition of Problem ============
    let delta=rand::thread_rng().gen_range(3..5);
    let mut Nred: Vec<(i32, i32)> = Vec::new();
    for swap_range in 1..delta+1{
        Nred.append((0..N-swap_range).map(|i| (i,i+swap_range)).collect());
    }


    let InitialSolutions :Vec<Vec<i32>> = vec![(0..N).collect()];
    //because Definition is like that, but there are more initials (4.1.)
    Algorithm1(&Nred, &InitialSolutions[0], &E);
}

fn Algorithm1(Nred: &Vec<(i32, i32)>, sched: &Vec<i32>, edges:&Vec<Vec<bool>>) -> Vec<(i32, i32)> {
    let mut moves=Nred.clone();
    /*
    //removes unfeasible(5)
    for i in 0..moves.len(){
        let u=moves[i].0;
        let v=moves[i].1;
        for x in u+1 .. v+1{
            if edges[sched[u]][sched[x]] {moves[i]=(0,0);break;}
        }
    }
    //moves (0,0) to the end
    let i=0;
    invalid=0;
    while i<moves.len()-invalid{
        if moves[i]==(0,0){
            while moves[moves.len()-invalid-1]==(0,0){
                invalid+=1;
            }
            if moves.len()-invalid-1<=i {break;}
            moves.swap(i,moves.len()-invalid-1);
            invalid+=1;
        }
    }
    //removes unfeasible(6)
    for i in 0..moves.len()-invalid{
        let u=moves[i].0;
        let v=moves[i].1;
        for x in u .. v{
            if edges[sched[x]][sched[v]] {moves[i]=(0,0);break;}
        }
    }
    //moves (0,0) to the end
    let i=0;
    // invalid=0; //no reset for invalids in the end
    while i<moves.len()-invalid{
        if moves[i]==(0,0){
            while moves[moves.len()-invalid-1]==(0,0){
                invalid+=1;
            }
            if moves.len()-invalid-1<=i {break;}
            moves.swap(i,moves.len()-invalid-1);
            invalid+=1;
        }
    }
    */
    //*
    moves.iter().filter(|(u,v)| {
        for x in u+1 .. v+1{
            if edges[sched[u]][sched[x]] {return false}
        }
        true
    });
    moves.iter().filter(|(u,v)| {
        for x in u .. v{
            if edges[sched[x]][sched[v]] {return false}
        }
        true
    });

    // */
    return moves
}
