extern crate num;
use num::bigint::BigInt;
use num::{FromPrimitive, ToPrimitive, One, Integer};

extern crate rayon;
use rayon::prelude::*;


extern crate itertools;
use itertools::Itertools;


use std::sync::{Arc, RwLock};
use std::collections::HashSet;

type SharedPrimes = Arc<RwLock<(Vec<BigInt>, HashSet<BigInt>)>>;

fn is_prime(num: &BigInt, known_primes: SharedPrimes) -> bool {
    if num.is_multiple_of(&BigInt::from_u32(2).unwrap()) || (num.is_multiple_of(&BigInt::from_u32(3).unwrap()) && num != &BigInt::from_u32(3).unwrap()){
        return false;
    }

    let limit = BigInt::from_f64(num.to_f64().unwrap().sqrt()).unwrap() + BigInt::one();
    let highest_prime;
    {
        let primes = known_primes.read().expect("Failed to acquire READ lock");
        highest_prime = if primes.0.len() != 0 {
            let not_prime = primes.0.iter()
                                  .filter(|known_prime| known_prime <= &&limit )
                                  .any(|known_prime| num.is_multiple_of(known_prime));
            if not_prime {
                return false;
            } else {
                Some(primes.0.iter().last().unwrap().clone())
            }
        } else {
            None
        }
    }
    let start = highest_prime.unwrap_or(BigInt::from_u32(3).unwrap());
    let limit = BigInt::from_f64(num.to_f64().unwrap().sqrt()).unwrap() + BigInt::one();
    let not_prime = num::range(start, limit)
                        .step(2)
                        .chunks(128 * 1024)
                        .into_iter()
                        .map(|chunk| {
                            chunk.collect::<Vec<BigInt>>()
                                .par_iter()
                                .any(|number| num.is_multiple_of(&number))
                        })
                        .any(|x| x);
    if !not_prime {
        let mut primes = known_primes.write().expect("Failed to acquire WRITE lock");
        //let (mut primes, mut prime_set) = known_primes.write().expect("Failed to acquire WRITE lock");
        if !primes.1.contains(&num) {
            primes.0.push(num.clone());
            primes.1.insert(num.clone());
            primes.0.sort();
        };
        true
    } else {
        false
    }



}

fn factorize(num: BigInt) -> Option<(BigInt, BigInt)> {
    let shared_primes = Arc::new(RwLock::new((vec![], HashSet::new())));
    let start = BigInt::from_u32(3).unwrap();
    let limit = BigInt::from_f64(num.to_f64().unwrap().sqrt()).unwrap();
    let range = num::range(start, limit + BigInt::one());
    let range_itr = range.into_iter().chunks(128 * 1024);
    let result = range_itr
                    .into_iter()
                    .filter_map(|chunk| {
                        chunk.collect::<Vec<BigInt>>()
                            .into_par_iter()
                            .filter_map(|div| {
                                if num.is_multiple_of(&div) {
                                    Some((div.clone(), (&num / div).clone()))
                                } else {
                                    None
                                }
                            })
                            .find_any(|&(ref div, ref prob_div)| {
                                    is_prime(&div, shared_primes.clone()) && is_prime(&prob_div, shared_primes.clone())
                            })
                    })
                    .next();
    {
        let primes = shared_primes.read().unwrap();
        println!("{:?}",primes.1.iter().map(|p| p).map(|p| format!("{}, ",p.to_str_radix(10))).collect::<Vec<String>>())
    }
    let n = 336703;
    println!("{} is prime -> {}",n, is_prime(&BigInt::from_u32(n).unwrap(), shared_primes.clone()) );
    let n = 370373;
    println!("{} is prime -> {}",n, is_prime(&BigInt::from_u32(n).unwrap(), shared_primes.clone()) );
    result
}


fn main() {
    let n = BigInt::parse_bytes(b"17969491597941066732916128449573246156367561808012600070888918835531726460341490933493372247868650755230855864199929221814436684722874052065257937495694348389263171152522525654410980819170611742509702440718010364831638288518852689", 10).unwrap();
    let n_small = BigInt::from_u32(11 * 13).unwrap();
    let n_bigger = BigInt::from_u32(370373).unwrap() * BigInt::from_u32(336703).unwrap();
    // println!("Factors for {:?} :\n {:?}", n, factorize(n.clone()));
    let factors: Vec<String> =
        factorize(n.clone())
            .iter()
            .map(|f| format!("{} * {}",BigInt::to_str_radix(&f.0, 10), BigInt::to_str_radix(&f.1, 10)))
            .collect();
    println!("Factors for {:?} : {:?}", n.to_str_radix(10), factors);

}
