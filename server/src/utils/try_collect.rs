/// Takes an iterator of results and collects it into a result without the wrapping results or
/// returns an Err as a whole if one of the items in the iterator is an Err.
pub fn try_collect<T, R, E>(iter: impl Iterator<Item = Result<T, E>>) -> Result<R, E>
where
    R: FromIterator<T>,
{
    let mut items: Vec<T> = vec![];

    for item in iter {
        items.push(item?)
    }

    Ok(items.into_iter().collect())
}
