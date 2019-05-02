

pub fn search<R, A, T>(root: T, reject: R, accept: A)
where
    T: Iterator<Item = T> + Send + Clone,
    R: Fn(&[T], &T) -> bool + Sync,
    A: Fn(&[T]) -> bool + Sync,
{
    let n = 8;
    let work = WorkGroup::<T>::new(n);
    crossbeam::scope(|scope| {
        for _ in 0..n {
            scope.spawn(|_| loop {
                let mut core = match work.solicit() {
                    Some(core) => core,
                    None => return,
                };
                let mut root_pointer = core.len() - 1;
                let bottom = root_pointer;
                loop {
                    match unsafe { core.get_unchecked_mut(root_pointer) }.next() {
                        Some(candidate) => {
                            if reject(&core[1..], &candidate) {
                                continue;
                            }
                            core.push(candidate);
                            if accept(&core[1..]) {
                                core.pop();
                                continue;
                            }
                            match work.acquire() {
                                true => {
                                    work.grant(core.clone());
                                    core.pop();
                                }
                                false => {
                                    root_pointer += 1;
                                }
                            }
                        }
                        None => {
                            core.pop();
                            if root_pointer == bottom {
                                work.quit();
                                break;
                            }
                            root_pointer -= 1;
                        }
                    }
                }
            });
        }
        work.acquire();
        work.grant(vec![root]);
        work.shutdown();
    })
    .unwrap();
}

type Core<T> = Vec<T>;

struct WorkGroup<T> {
    n: usize,
    s: crossbeam::channel::Sender<Option<Core<T>>>,
    r: crossbeam::channel::Receiver<Option<Core<T>>>,
    active_counter: crossbeam::atomic::AtomicCell<usize>,
    wg: WaitGroup
}

impl <T> WorkGroup<T> {
    pub fn new(n: usize) -> WorkGroup<T> {
        let (s, r) = crossbeam::channel::bounded(n);
        let active_counter = crossbeam::atomic::AtomicCell::new(0);
        let wg = WaitGroup::new(n);
        WorkGroup{n, s, r, active_counter, wg}
    }
    pub fn acquire(&self) -> bool {
        if self.active_counter.load() < self.n {
            if self.active_counter.fetch_add(1) >= self.n {
                // We got cutoff.
                self.active_counter.fetch_sub(1);
                false
            } else {
                self.wg.add(1);
                true
            }
        } else {
            false
        }
    }
    pub fn shutdown(&self) {
        self.wg.wait();
        for _ in 0..self.n {
            self.s.send(None).unwrap();
        }
    }
    pub fn quit(&self) {
        self.wg.done();
        self.active_counter.fetch_sub(1);
    }
    pub fn solicit(&self) -> Option<Core<T>> {
        self.r.recv().unwrap()
    }
    pub fn grant(&self, core: Core<T>) {
        self.s.send(Some(core)).unwrap();
    }
}

struct WaitGroup {
    s: crossbeam::channel::Sender<isize>,
    r: crossbeam::channel::Receiver<isize>
}

impl WaitGroup {
    pub fn new(cap: usize) -> WaitGroup {
        let (s, r) = crossbeam::channel::bounded(cap);
        WaitGroup{s, r}
    }
    pub fn add(&self, i: isize) {
        self.s.send(i).unwrap();
    }
    pub fn done(&self, ) {
        self.s.send(-1).unwrap();
    }
    pub fn wait(&self) {
        // Believe it or not, this is marginally faster than crossbeam::sync::WaitGroup
        let mut alive = 0;
        loop {
            alive += self.r.recv().unwrap();
            if alive == 0 {
                break;
            }
        }
    }
}