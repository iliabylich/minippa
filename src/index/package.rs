#[derive(Debug)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) uploaded_at: String,

    pub(crate) full: String,
}
