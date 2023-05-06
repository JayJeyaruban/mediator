pub trait IntoLabelled<T> {
    fn with_label<L: AsRef<str>>(self, label: L) -> Labelled<L, T>;
}

impl<T> IntoLabelled<T> for T {
    fn with_label<L: AsRef<str>>(self, label: L) -> Labelled<L, T> {
        Labelled { label, inner: self }
    }
}

pub struct Labelled<L, T>
where
    L: AsRef<str>,
{
    pub label: L,
    pub inner: T,
}
