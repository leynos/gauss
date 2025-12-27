pub enum Command {
    /// Delete the specified shapes.
    DeleteShapes {
        /// Shape IDs to delete, with their indices and data for undo.
        targets: Vec<DeletedShape>,
    },
    // Future commands:
    // MoveShapes { shape_ids: Vec<ShapeId>, delta: Vec2 },
    // SetStyle { shape_ids: Vec<ShapeId>, from: PaintStyle, to: PaintStyle },
    // InsertShape { index: usize, shape: Shape },
    // ...
}
