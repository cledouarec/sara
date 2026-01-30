//! Traceability matrix generation.

use serde::Serialize;

use crate::graph::KnowledgeGraph;
use crate::model::ItemType;

/// A row in the traceability matrix.
#[derive(Debug, Clone, Serialize)]
pub struct MatrixRow {
    /// Source item ID.
    pub source_id: String,
    /// Source item name.
    pub source_name: String,
    /// Source item type.
    pub source_type: String,
    /// Target item IDs (relationships).
    pub targets: Vec<MatrixTarget>,
}

/// A target in the traceability matrix.
#[derive(Debug, Clone, Serialize)]
pub struct MatrixTarget {
    /// Target item ID.
    pub id: String,
    /// Target item name.
    pub name: String,
    /// Target item type.
    pub target_type: String,
    /// Relationship type.
    pub relationship: String,
}

/// The complete traceability matrix.
#[derive(Debug, Clone, Serialize)]
pub struct TraceabilityMatrix {
    /// Matrix rows (one per source item).
    pub rows: Vec<MatrixRow>,
    /// Column headers (item types).
    pub columns: Vec<String>,
    /// Total number of relationships.
    pub total_relationships: usize,
}

impl TraceabilityMatrix {
    /// Generates a traceability matrix from a knowledge graph.
    pub fn generate(graph: &KnowledgeGraph) -> Self {
        let mut rows: Vec<MatrixRow> = graph
            .items()
            .map(|item| Self::build_row(item, graph))
            .collect();

        let total_relationships = rows.iter().map(|r| r.targets.len()).sum();

        Self::sort_rows(&mut rows);

        let columns = Self::build_columns();

        Self {
            rows,
            columns,
            total_relationships,
        }
    }

    /// Builds a matrix row for an item.
    fn build_row(item: &crate::model::Item, graph: &KnowledgeGraph) -> MatrixRow {
        let mut targets = Vec::new();

        Self::collect_upstream_targets(item, graph, &mut targets);
        Self::collect_downstream_targets(item, graph, &mut targets);

        MatrixRow {
            source_id: item.id.as_str().to_string(),
            source_name: item.name.clone(),
            source_type: item.item_type.display_name().to_string(),
            targets,
        }
    }

    /// Collects upstream relationship targets.
    fn collect_upstream_targets(
        item: &crate::model::Item,
        graph: &KnowledgeGraph,
        targets: &mut Vec<MatrixTarget>,
    ) {
        for rel in item.upstream_relationships() {
            if let Some(target) = graph.get(&rel.to) {
                targets.push(MatrixTarget {
                    id: rel.to.as_str().to_string(),
                    name: target.name.clone(),
                    target_type: target.item_type.display_name().to_string(),
                    relationship: rel.relationship_type.field_name().as_str().to_string(),
                });
            }
        }
    }

    /// Collects downstream relationship targets.
    fn collect_downstream_targets(
        item: &crate::model::Item,
        graph: &KnowledgeGraph,
        targets: &mut Vec<MatrixTarget>,
    ) {
        for rel in item.downstream_relationships() {
            if let Some(target) = graph.get(&rel.to) {
                targets.push(MatrixTarget {
                    id: rel.to.as_str().to_string(),
                    name: target.name.clone(),
                    target_type: target.item_type.display_name().to_string(),
                    relationship: rel.relationship_type.field_name().as_str().to_string(),
                });
            }
        }
    }

    /// Sorts rows by type order, then by ID.
    fn sort_rows(rows: &mut [MatrixRow]) {
        rows.sort_by(|a, b| {
            let type_order_a = Self::type_order(&a.source_type);
            let type_order_b = Self::type_order(&b.source_type);
            type_order_a
                .cmp(&type_order_b)
                .then(a.source_id.cmp(&b.source_id))
        });
    }

    /// Builds column headers from item types.
    fn build_columns() -> Vec<String> {
        ItemType::all()
            .iter()
            .map(|t| t.display_name().to_string())
            .collect()
    }

    /// Returns the type order for sorting.
    fn type_order(type_name: &str) -> usize {
        match type_name {
            "Solution" => 0,
            "Use Case" => 1,
            "Scenario" => 2,
            "System Requirement" => 3,
            "System Architecture" => 4,
            "Hardware Requirement" => 5,
            "Software Requirement" => 6,
            "Hardware Detailed Design" => 7,
            "Software Detailed Design" => 8,
            _ => 9,
        }
    }

    /// Converts the matrix to CSV format.
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();

        csv.push_str(
            "Source ID,Source Name,Source Type,Target ID,Target Name,Target Type,Relationship\n",
        );

        for row in &self.rows {
            if row.targets.is_empty() {
                csv.push_str(&format!(
                    "{},{},{},,,, \n",
                    Self::escape_csv(&row.source_id),
                    Self::escape_csv(&row.source_name),
                    Self::escape_csv(&row.source_type),
                ));
            } else {
                for target in &row.targets {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        Self::escape_csv(&row.source_id),
                        Self::escape_csv(&row.source_name),
                        Self::escape_csv(&row.source_type),
                        Self::escape_csv(&target.id),
                        Self::escape_csv(&target.name),
                        Self::escape_csv(&target.target_type),
                        Self::escape_csv(&target.relationship),
                    ));
                }
            }
        }

        csv
    }

    /// Escapes a value for CSV output.
    fn escape_csv(value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;
    use crate::model::{ItemId, RelationshipType};
    use crate::test_utils::{create_test_item, create_test_item_with_relationships};

    #[test]
    fn test_matrix_generation() {
        let sol = create_test_item("SOL-001", ItemType::Solution);
        let uc = create_test_item_with_relationships(
            "UC-001",
            ItemType::UseCase,
            vec![(ItemId::new_unchecked("SOL-001"), RelationshipType::Refines)],
        );

        let graph = GraphBuilder::new()
            .add_item(sol)
            .add_item(uc)
            .build()
            .unwrap();

        let matrix = TraceabilityMatrix::generate(&graph);
        assert_eq!(matrix.rows.len(), 2);
        assert!(matrix.total_relationships > 0);
    }

    #[test]
    fn test_matrix_csv() {
        let sol = create_test_item("SOL-001", ItemType::Solution);

        let graph = GraphBuilder::new().add_item(sol).build().unwrap();

        let matrix = TraceabilityMatrix::generate(&graph);
        let csv = matrix.to_csv();
        assert!(csv.contains("Source ID"));
        assert!(csv.contains("SOL-001"));
    }
}
