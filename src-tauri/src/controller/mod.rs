pub mod anki;

#[macro_export]
macro_rules! controller_handlers {
    () => {
        $crate::anki_handlers!()
    };
}
