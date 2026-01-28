use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PageId(String);

impl PageId {
    pub(crate) fn new(rel: &Path) -> Self {
        Self(rel.with_extension("").to_string_lossy().into_owned())
    }

    pub(crate) fn root_of(id: &Self) -> Self {
        PageId(id.0.split('/').next().unwrap_or("").to_string())
    }

    pub(crate) fn as_ref(&self) -> PageIdRef<'_> {
        PageIdRef(&self.0)
    }
}

/// Maybe some day I will wrap my head around of zero-copy indexes, or whatever i'll need this for.
pub(crate) struct PageIdRef<'a>(&'a str);

impl std::fmt::Display for PageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<P> From<P> for PageId
where
    P: AsRef<Path>,
{
    fn from(value: P) -> Self {
        Self(
            value
                .as_ref()
                .with_extension("")
                .to_string_lossy()
                .into_owned(),
        )
    }
}
