use crossbeam::epoch;

//https://aturon.github.io/blog/2015/08/27/epoch/

struct Tree {
    collector: epoch::Collector,
}

impl Tree {
    pub fn new() {
        let c = epoch::Collector::new();
    }
}