use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphProperty {
    Text(String),
    Number(f64),
    Boolean(bool),
    TextList(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub properties: BTreeMap<String, GraphProperty>,
}

impl GraphNode {
    pub fn new(id: impl Into<String>, node_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            node_type: node_type.into(),
            properties: BTreeMap::new(),
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: GraphProperty) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source_id: String,
    pub target_id: String,
    pub relationship: String,
    pub properties: BTreeMap<String, GraphProperty>,
}

impl GraphEdge {
    pub fn new(
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        relationship: impl Into<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            relationship: relationship.into(),
            properties: BTreeMap::new(),
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: GraphProperty) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(self, _node: GraphNode) -> Self {
        self
    }

    pub fn add_edge(self, _edge: GraphEdge) -> Self {
        self
    }

    pub fn node(&self, _id: &str) -> Option<&GraphNode> {
        None
    }

    pub fn nodes_by_type(&self, _node_type: &str) -> Vec<&GraphNode> {
        Vec::new()
    }

    pub fn edges_by_relationship(&self, _relationship: &str) -> Vec<&GraphEdge> {
        Vec::new()
    }

    pub fn neighbors(&self, _source_id: &str, _relationship: &str) -> Vec<&GraphNode> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_graph() -> KnowledgeGraph {
        KnowledgeGraph::new()
            .add_node(
                GraphNode::new("doc-1", "document")
                    .with_property("title", GraphProperty::Text("RAG Guide".to_string()))
                    .with_property("page_count", GraphProperty::Number(3.0))
                    .with_property("trusted", GraphProperty::Boolean(true)),
            )
            .add_node(
                GraphNode::new("chunk-1", "chunk")
                    .with_property(
                        "text",
                        GraphProperty::Text("RAG evaluates retrieval".to_string()),
                    )
                    .with_property(
                        "entities",
                        GraphProperty::TextList(vec!["RAG".to_string(), "retrieval".to_string()]),
                    ),
            )
            .add_node(GraphNode::new("chunk-2", "chunk").with_property(
                "text",
                GraphProperty::Text("LLM judges score answers".to_string()),
            ))
            .add_edge(
                GraphEdge::new("doc-1", "chunk-1", "contains")
                    .with_property("order", GraphProperty::Number(1.0)),
            )
            .add_edge(
                GraphEdge::new("doc-1", "chunk-2", "contains")
                    .with_property("order", GraphProperty::Number(2.0)),
            )
    }

    #[test]
    fn test_13_1_1_graph_stores_nodes_relationships_and_typed_properties() {
        // SCEN-13.1.1 / AC1 / TEST-13.1.1
        let graph = fixture_graph();

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(
            graph
                .node("doc-1")
                .and_then(|node| node.properties.get("trusted")),
            Some(&GraphProperty::Boolean(true))
        );
        assert_eq!(
            graph.edges[0].properties.get("order"),
            Some(&GraphProperty::Number(1.0))
        );
    }

    #[test]
    fn test_13_1_2_graph_queries_filter_by_type_and_relationship() {
        // SCEN-13.1.2 / AC2 / TEST-13.1.2
        let graph = fixture_graph();

        let chunks = graph.nodes_by_type("chunk");
        assert_eq!(chunks.len(), 2);
        assert!(chunks.iter().all(|node| node.node_type == "chunk"));

        let contains = graph.edges_by_relationship("contains");
        assert_eq!(contains.len(), 2);

        let neighbors = graph.neighbors("doc-1", "contains");
        let neighbor_ids: Vec<&str> = neighbors.iter().map(|node| node.id.as_str()).collect();
        assert_eq!(neighbor_ids, vec!["chunk-1", "chunk-2"]);
    }

    #[test]
    fn test_13_1_3_graph_serialization_roundtrips_fixtures() {
        // SCEN-13.1.3 / AC3 / TEST-13.1.3
        let graph = fixture_graph();

        let json = serde_json::to_string(&graph).expect("serialize graph");
        assert!(json.contains("\"relationship\":\"contains\""));
        assert!(json.contains("\"TextList\""));

        let roundtrip: KnowledgeGraph = serde_json::from_str(&json).expect("deserialize graph");
        assert_eq!(roundtrip, graph);
        assert_eq!(roundtrip.nodes_by_type("document")[0].id, "doc-1");
    }
}
