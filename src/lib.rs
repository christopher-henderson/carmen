#![feature(test)]
extern crate crossbeam;

pub mod sequential;
pub mod parallel;

#[cfg(test)]
mod tests {

    extern crate test;
    extern crate time;

    use super::*;
    use test::Bencher;

    #[derive(Debug, Clone)]
    pub struct Queen {
        pub column: i32,
        pub row: i32,
        pub n: i32,
        current: i32,
    }

    impl Queen {
        pub fn new(column: i32, row: i32, n: i32) -> Queen {
            Queen {
                column,
                row,
                n,
                current: 0,
            }
        }
    }

    impl Iterator for Queen {
        type Item = Queen;

        fn next(&mut self) -> Option<Queen> {
            self.current += 1;
            if self.current > self.n {
                return None;
            }
            Some(Queen::new(self.column + 1, self.current, self.n))
        }
    }

    #[test]
    fn base_par() {
        let answers = crossbeam::atomic::AtomicCell::<usize>::new(0);
        let n = 10;
        parallel::search(
            Queen::new(0, 0, n),
            |solution, candidate| {
                let column = candidate.column;
                let row = candidate.row;
                for queen in solution.iter() {
                    let c = queen.column;
                    let r = queen.row;
                    if (row == r)
                        || (column == c)
                        || (row + column == r + c)
                        || (row - column == r - c)
                    {
                        return true;
                    }
                }
                return false;
            },
            |solution| {
                if solution.len() > 0 && solution.len() == n as usize {
                    answers.fetch_add(1);
                    return true;
                }
                return false;
            },
        );
//        assert_eq!(14772512, answers.load());
    }

    #[test]
    fn base_seq() {
        let answers = crossbeam::atomic::AtomicCell::<usize>::new(0);
        let n = 10;
        sequential::search(
            Queen::new(0, 0, n),
            |solution, candidate| {
                let column = candidate.column;
                let row = candidate.row;
                for queen in solution.iter() {
                    let c = queen.column;
                    let r = queen.row;
                    if (row == r)
                        || (column == c)
                        || (row + column == r + c)
                        || (row - column == r - c)
                    {
                        return true;
                    }
                }
                return false;
            },
            |solution| {
                if solution.len() > 0 && solution.len() == n as usize {
                    answers.fetch_add(1);
                    return true;
                }
                return false;
            },
        );
//        assert_eq!(92, answers.load());
    }

    #[test]
    fn comp() {
        let start = time::PreciseTime::now();
        base_seq();
        let end = time::PreciseTime::now();
        let seq = start.to(end);
        let start = time::PreciseTime::now();
        base_par();
        let end = time::PreciseTime::now();
        let par = start.to(end);
        eprintln!("sequential = {:#?}", seq);
        eprintln!("parallel = {:#?}", par);
    }

    #[bench]
    fn base_bench(b: &mut Bencher) {
        b.iter(|| {
            let n = 8;
            parallel::search(
                Queen::new(0, 0, n),
                |solution, candidate| {
                    let column = candidate.column;
                    let row = candidate.row;
                    for queen in solution.iter() {
                        let c = queen.column;
                        let r = queen.row;
                        if (row == r)
                            || (column == c)
                            || (row + column == r + c)
                            || (row - column == r - c)
                        {
                            return true;
                        }
                    }
                    return false;
                },
                |solution| solution.len() > 0 && solution.len() == n as usize,
            );
        });
    }
}
