use std::borrow::Borrow;

pub trait IntoLabelled<T> {
    fn labelled<L>(self, label: L) -> Labelled<L, T>;
}

impl<T> IntoLabelled<T> for T {
    fn labelled<L>(self, label: L) -> Labelled<L, T> {
        Labelled { label, inner: self }
    }
}

pub struct Labelled<L, T: ?Sized> {
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

pub trait FindByLabel<L1, L2, T: ?Sized> {
    fn find_by_label(&self, label: L1) -> Option<&Labelled<L2, T>>;
}

impl<L, L1, L2, T> FindByLabel<L1, L2, T> for Vec<L>
where
    L: Borrow<Labelled<L2, T>>,
    L1: PartialEq<L2>,
    L2: PartialEq<L1>,
{
    fn find_by_label(&self, label: L1) -> Option<&Labelled<L2, T>> {
        self.iter()
            .map(|item| item.borrow())
            .find(|item| item.label == label)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::util::label::FindByLabel;

    use super::IntoLabelled;

    #[test]
    fn findby() {
        let list = vec!["world".labelled("hello"), "second".labelled("the")];
        let result = list.find_by_label("hello");

        assert!(result.is_some() && result.unwrap().inner == "world");
    }

    #[test]
    fn findby_borrow() {
        let item1 = "world".labelled("hello");
        let item2 = "second".labelled("the");
        let list = vec![&item1, &item2];
        let result = list.find_by_label("hello");

        assert!(result.is_some() && result.unwrap().inner == "world");
    }

    #[test]
    fn findby_box() {
        let item1 = Box::new("world".labelled("hello"));
        let item2 = Box::new("second".labelled("the"));
        let list = vec![item1, item2];
        let result = list.find_by_label("hello");

        assert!(result.is_some() && result.unwrap().inner == "world");
    }

    #[test]
    fn findby_arc() {
        let item1 = Arc::new("world".labelled("hello"));
        let item2 = Arc::new("second".labelled("the"));
        let list = vec![item1.clone(), item2.clone()];
        let result = list.find_by_label("hello");

        assert!(result.is_some() && result.unwrap().inner == "world");
    }
}
