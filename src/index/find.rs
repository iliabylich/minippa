use crate::index::{Package, list::list};
use anyhow::{Result, bail};
use std::collections::HashSet;

pub(crate) async fn find(dir: &str, name: String) -> Result<Option<Package>> {
    let filters = HashSet::from([name]);
    let mut iter = list(dir, Some(filters)).await?.into_iter();

    let Some(package) = iter.next() else {
        return Ok(None);
    };
    if iter.next().is_some() {
        bail!("multiple packages found");
    }
    Ok(Some(package))
}
