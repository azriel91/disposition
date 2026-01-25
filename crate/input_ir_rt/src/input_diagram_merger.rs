use disposition_input_model::InputDiagram;

/// Merges an input diagram over another.
#[derive(Clone, Copy, Debug)]
pub struct InputDiagramMerger;

impl InputDiagramMerger {
    pub fn merge<'f, 'id>(
        base_diagram: InputDiagram<'static>,
        overlay_diagram: &'f InputDiagram<'id>,
    ) -> InputDiagram<'id>
    where
        'id: 'f,
    {
        todo!()
    }
}
