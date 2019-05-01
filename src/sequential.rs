pub fn search<R, A, T>(root: T, reject: R, accept: A)
where
    T: Iterator<Item = T> + Clone,
    R: Fn(&[T], &T) -> bool,
    A: Fn(&[T]) -> bool,
{
    let mut root_pointer: usize = 0;
    let mut core = vec![root];
    loop {
        if let Some(candidate) = unsafe { core.get_unchecked_mut(root_pointer) }.next() {
            if reject(&core[1..], &candidate) {
                continue;
            }
            core.push(candidate);
            if accept(&core[1..]) {
                core.pop();
                continue;
            }
            root_pointer += 1;
        } else {
            core.pop();
            if root_pointer == 0 {
                break;
            }
            root_pointer -= 1;
        }
    }
}
