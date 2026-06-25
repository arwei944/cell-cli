use crate::domain::cell_spec::CellSpec;

pub trait CodeGeneratorPort {
    fn render_cell_structure(&self, spec: &CellSpec) -> Vec<GeneratedFile>;
}

#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}
