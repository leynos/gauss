#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Document, Selection, SelItem, Shape};
    use rstest::{fixture, rstest};

    #[fixture]
    fn empty_doc() -> Document {
        Document::default()
    }

    #[fixture]
    fn doc_with_two_shapes() -> Document {
        let mut doc = Document::default();
        doc.shapes.push(Shape::default());
        doc.shapes.push(Shape::default());
        doc
    }

    #[rstest]
    fn delete_shapes_removes_from_document(mut doc_with_two_shapes: Document) {
        let cmd = Command::DeleteShapes {
            targets: vec![DeletedShape {
                index: 0,
                shape: doc_with_two_shapes.shapes[0].clone(),
            }],
        };

        let result = cmd.apply(&mut doc_with_two_shapes);
        assert!(result.is_ok());
        assert_eq!(doc_with_two_shapes.shapes.len(), 1);
    }

    #[rstest]
    fn delete_shapes_inverse_restores(mut doc_with_two_shapes: Document) {
        let original_len = doc_with_two_shapes.shapes.len();
        let shape = doc_with_two_shapes.shapes[0].clone();

        let cmd = Command::DeleteShapes {
            targets: vec![DeletedShape { index: 0, shape }],
        };

        let inverse = cmd.apply(&mut doc_with_two_shapes).expect("apply succeeded");
        assert_eq!(doc_with_two_shapes.shapes.len(), original_len - 1);

        inverse.apply(&mut doc_with_two_shapes).expect("undo succeeded");
        assert_eq!(doc_with_two_shapes.shapes.len(), original_len);
    }

    #[rstest]
    fn prepare_delete_selection_fails_with_empty_selection(doc_with_two_shapes: Document) {
        let selection = Selection::default();

        let result = prepare_command(
            Action::DeleteSelection,
            &doc_with_two_shapes,
            &selection,
        );

        assert!(matches!(result, Err(CommandError::EmptySelection)));
    }

    #[rstest]
    fn command_name_is_nonempty() {
        let cmd = Command::DeleteShapes { targets: vec![] };
        assert!(!cmd.name().is_empty());
    }
}
