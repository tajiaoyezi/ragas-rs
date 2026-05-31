use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocExample {
    pub workflow: String,
    pub example_path: String,
    pub upstream_section: String,
    pub feature_flags: Vec<String>,
    pub known_parity_gaps: Vec<String>,
}

pub fn public_workflow_examples() -> Vec<DocExample> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeSet, path::Path};

    #[test]
    fn test_16_2_1_each_public_workflow_has_a_runnable_rust_example() {
        // SCEN-16.2.1 / AC1 / TEST-16.2.1
        let examples = public_workflow_examples();
        let workflows = examples
            .iter()
            .map(|example| example.workflow.as_str())
            .collect::<BTreeSet<_>>();

        assert!(workflows.contains("evaluate"));
        assert!(workflows.contains("testset"));
        assert!(workflows.contains("benchmark"));
        for example in &examples {
            assert!(
                Path::new(&example.example_path).exists(),
                "{} should exist",
                example.example_path
            );
            assert!(
                example.example_path.ends_with(".rs"),
                "{} should be a Rust example",
                example.example_path
            );
        }
    }

    #[test]
    fn test_16_2_2_examples_map_to_upstream_docs_section_names() {
        // SCEN-16.2.2 / AC2 / TEST-16.2.2
        let sections = public_workflow_examples()
            .into_iter()
            .map(|example| example.upstream_section)
            .collect::<BTreeSet<_>>();

        assert!(sections.contains("Evaluate a RAG application"));
        assert!(sections.contains("Generate a testset"));
        assert!(sections.contains("Compare and monitor evaluation cost"));
    }

    #[test]
    fn test_16_2_3_docs_state_feature_flags_and_known_parity_gaps() {
        // SCEN-16.2.3 / AC3 / TEST-16.2.3
        let docs = std::fs::read_to_string("docs/ragas-rs-user-guide.md").expect("user guide");

        assert!(docs.contains("Feature flags"));
        assert!(docs.contains("Known parity gaps"));
        assert!(docs.contains("no Python runtime bridge"));
        assert!(docs.contains("knowledge graph generation"));
        assert!(
            public_workflow_examples()
                .iter()
                .any(|example| example.feature_flags.contains(&"default".to_string()))
        );
    }
}
