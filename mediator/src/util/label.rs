pub trait IntoLabelled<T> {
    fn labelled<L: AsRef<str>>(self, label: L) -> Labelled<L, T>;
}

impl<T> IntoLabelled<T> for T {
    fn labelled<L: AsRef<str>>(self, label: L) -> Labelled<L, T> {
        Labelled { label, inner: self }
    }
}

pub struct Labelled<L, T> {
    pub label: L,
    pub inner: T,
}

impl<L, T> Labelled<L, T> {
    pub fn with_label<NewLabel>(self, new_label: NewLabel) -> Labelled<NewLabel, T> {
        Labelled {
            label: new_label,
            inner: self.inner,
        }
    }
}
