type WorkSender<T> = crossbeam::channel::Sender<Option<Vec<T>>>;
type WorkReceiver<T> = crossbeam::channel::Receiver<Option<Vec<T>>>;

pub fn search<R, A, T>(root: T, reject: R, accept: A)
where
    T: Iterator<Item = T> + Send + Clone,
    R: Fn(&[T], &T) -> bool + Sync,
    A: Fn(&[T]) -> bool + Sync,
{
    let n = 8;
    let (wgs, wgr) = crossbeam::channel::bounded(n);
    let (aws, awr) = crossbeam::channel::bounded(n);
    let (ws, wr): (WorkSender<T>, WorkReceiver<T>) = crossbeam::channel::bounded(n);
    crossbeam::scope(|scope| {
        for _ in 0..n {
            scope.spawn(|_| loop {
                let mut core = match wr.recv() {
                    Ok(Some(core)) => core,
                    Ok(None) => return,
                    Err(err) => panic!(err),
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
                            match aws.try_send(1) {
                                Ok(_) => {
                                    wgs.send(1).unwrap();
                                    ws.send(Some(core.clone())).unwrap();
                                    core.pop();
                                }
                                Err(crossbeam::channel::TrySendError::Full(_)) => {
                                    root_pointer += 1;
                                }
                                Err(crossbeam::channel::TrySendError::Disconnected(err)) => {
                                    panic!(err)
                                }
                            }
                        }
                        None => {
                            core.pop();
                            if root_pointer == bottom {
                                wgs.send(-1).unwrap();
                                awr.recv().unwrap();
                                break;
                            }
                            root_pointer -= 1;
                        }
                    }
                }
            });
        }
        wgs.send(1).unwrap();
        aws.send(1).unwrap();
        ws.send(Some(vec![root])).unwrap();
        let mut alive = 0;
        loop {
            alive += wgr.recv().unwrap();
            if alive == 0 {
                break;
            }
        }
        for _ in 0..n {
            ws.send(None).unwrap();
        }
    })
    .unwrap();
}
