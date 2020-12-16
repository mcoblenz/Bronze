use bronze::*;
use crate::document::*;

// This is the view side of a document.

pub struct DocumentView {
    document: GcHandle<Document>,
}