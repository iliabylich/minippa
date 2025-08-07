use crate::index::Package;
use askama::Template;

#[derive(Template)]
#[template(path = "list.html")]
pub(crate) struct List {
    pub(crate) packages: Vec<Package>,
}

#[derive(Template)]
#[template(path = "one.html")]
pub(crate) struct One {
    pub(crate) package: Package,
}
