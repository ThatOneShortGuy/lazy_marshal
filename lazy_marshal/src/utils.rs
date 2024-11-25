pub(crate) fn readn_to_vec(
    data: &mut impl Iterator<Item = u8>,
    n: usize,
) -> Result<Vec<u8>, usize> {
    let v = (0..n).fold(Vec::with_capacity(n), |mut acc, _| match data.next() {
        Some(d) => {
            acc.push(d);
            acc
        }
        None => acc,
    });

    if v.len() < n {
        Err(v.len())
    } else {
        Ok(v)
    }
}
