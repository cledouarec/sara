//! Template generation for frontmatter and documents.

mod generator;

pub use generator::{
    GeneratorOptions, extract_name_from_content, generate_document, generate_id, suggest_next_id,
};
