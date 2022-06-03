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
                else {
                    let t :i32=rand::thread_rng().gen_range(0..3);
                    if t==0{true}
                    else {false}
                } //chance is 1/3 here for edges
            }).collect()
        }).collect();
    //number of ressources
    let M = rand::thread_rng().gen_range(3..7);
    //maximal resources of system
    let Rmax :Vec<i32> = (0..M).map(|_| rand::thread_rng().gen_range(6..14)).collect();
    //ressource matrix
    let R :Vec<Vec<i32>> = (0..N).map(|_| {
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
    let mut Nred: Vec<(usize, usize)> = Vec::new();
    for swap_range in 1..delta+1{
        let mut temp : Vec<(usize, usize)>=(0 as usize..(N-swap_range) as usize).map(|i| (i,i+swap_range as usize)).collect();
        Nred.append(&mut temp);
    }
    let tabuList_len=100;

    let InitialSolutions :Vec<Vec<i32>> = vec![(0..N).collect()];
    //because Definition is like that, but there are more initials (4.1.)

    {
        let mut tabuList:Vec<(usize,usize)>=vec![(0,0); tabuList_len];
        let mut tabuCache:Vec<Vec<bool>> = vec![vec![false;N as usize];N as usize];
        let mut writeIndex : usize = 0;
        {
            let Nfeasable: Vec<(usize, usize)> = generate_Nred(&Nred, &InitialSolutions[0], &E);

            if Nfeasable[0]!=(0,0) && !check_STL(Nfeasable[0],&tabuCache){
                addTo_STL(Nfeasable[0], &mut tabuCache, &mut tabuList, &mut writeIndex);
            }
        }
    }



    Ok(())
}

//Algorithm1
fn generate_Nred(Nred: &Vec<(usize, usize)>, sched: &Vec<i32>, edges:&Vec<Vec<bool>>) -> Vec<(usize, usize)> {
    let mut moves: Vec<(usize, usize)>=Nred.clone();
    let moves_len=moves.len();
    //*
    //removes unfeasible(5)
    for i in 0..moves_len{
        let u=moves[i].0;
        let v=moves[i].1;
        for x in u+1 .. v+1{
            if edges
                [sched[u] as usize]
                [sched[x as usize] as usize]
            {moves[i]=(0,0);break;}
        }
    }
    //moves (0,0) to the end
    let mut i=0;
    let mut invalid=0;
    while i<moves_len-invalid{
        if moves[i]==(0,0){
            while moves[moves_len-invalid-1]==(0,0){
                invalid+=1;
            }
            if moves_len-invalid-1<=i {break;}
            moves.swap(i,moves_len-invalid-1);
            invalid+=1;
        }
        i+=1;
    }
    //removes unfeasible(6)
    for i in 0..moves_len-invalid{
        let u=moves[i].0;
        let v=moves[i].1;
        for x in u .. v{
            if edges
                [sched[x] as usize]
                [sched[v] as usize]
            {moves[i]=(0,0);break;}
        }
    }
    //moves (0,0) to the end
    i=0;
    // invalid=0; //no reset for invalids in the end
    while i<moves_len-invalid{
        if moves[i]==(0,0){
            while moves[moves_len-invalid-1]==(0,0){
                invalid+=1;
            }
            if moves_len-invalid-1<=i {break;}
            moves.swap(i,moves_len-invalid-1);
            invalid+=1;
        }
        i+=1;
    }
    //*/
    /*
    moves=moves.into_iter().filter(|(u,v)| {
        for x in u+1 .. v+1{
            if edges
                [sched[*u] as usize]
                [sched[x] as usize]
            {return false}
        }
        true
    }).collect();
    moves=moves.into_iter().filter(|(u,v)| {
        for x in *u .. *v{
            if edges
                [sched[x] as usize]
                [sched[*v] as usize]
            {return false}
        }
        true
    }).collect();
    //let t = sched.get(5);
    // */
    return moves
}

//Algorithm2
fn check_STL((u,v):(usize, usize),tabuCache:&Vec<Vec<bool>>)->bool{
    tabuCache[u][v].clone()
}

//Algorithm3
fn addTo_STL((u,v):(usize, usize), tabuCache:&mut Vec<Vec<bool>>, tabuList:&mut Vec<(usize,usize)>, writeIndex: &mut usize){
    let (uold, vold) = tabuList[*writeIndex].clone();
    tabuCache[uold][vold] = false;
    tabuList[*writeIndex]= (u, v).clone();
    tabuCache[u][v] = true;
    *writeIndex = (*writeIndex + 1) % tabuList.len();

}