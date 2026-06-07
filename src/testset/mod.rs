use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    ChatMessage, DistanceMeasure, EmbeddingProvider, EmbeddingRequest, EvaluationDataset,
    LlmProvider, LlmRequest, RagasError, SingleTurnSample, cosine_similarity,
    string_distance_similarity_with,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphProperty {
    Text(String),
    Number(f64),
    Boolean(bool),
    TextList(Vec<String>),
    /// A dense embedding vector (e.g. produced by [`EmbeddingExtractor`] and consumed by
    /// [`build_cosine_relationships`]). Stored as `f32` to match the provider output and the
    /// [`crate::cosine_similarity`] signature.
    Vector(Vec<f32>),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphParityFixture {
    pub feature: String,
    pub upstream_commit: String,
    pub graph: KnowledgeGraph,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphCluster {
    pub key: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphAdvancedQuery {
    pub node_type: Option<String>,
    pub property_key: Option<String>,
    pub property_value: Option<GraphProperty>,
    pub outgoing_relationship: Option<String>,
}

impl Default for GraphAdvancedQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphAdvancedQuery {
    pub fn new() -> Self {
        Self {
            node_type: None,
            property_key: None,
            property_value: None,
            outgoing_relationship: None,
        }
    }

    pub fn with_node_type(mut self, node_type: impl Into<String>) -> Self {
        self.node_type = Some(node_type.into());
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: GraphProperty) -> Self {
        self.property_key = Some(key.into());
        self.property_value = Some(value);
        self
    }

    pub fn with_outgoing_relationship(mut self, relationship: impl Into<String>) -> Self {
        self.outgoing_relationship = Some(relationship.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SynthesizerStrategy {
    SingleHop,
    MultiHop,
    PreChunked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynthesizerPromptMessage {
    pub role: String,
    pub template: String,
}

impl SynthesizerPromptMessage {
    pub fn new(role: impl Into<String>, template: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            template: template.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynthesizerPromptSnapshot {
    pub strategy: SynthesizerStrategy,
    pub variables: BTreeMap<String, String>,
    pub messages: Vec<SynthesizerPromptMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedSynthesizerPromptMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedSynthesizerPromptSnapshot {
    pub strategy: SynthesizerStrategy,
    pub variables: BTreeMap<String, String>,
    pub rendered_messages: Vec<RenderedSynthesizerPromptMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesizerSampleComparison {
    pub matches: bool,
    pub drift: Option<String>,
}

pub fn parse_graph_parity_fixture(input: &str) -> Result<GraphParityFixture, RagasError> {
    serde_json::from_str(input).map_err(|error| RagasError::Parse {
        message: format!("graph parity fixture parse failed: {error}"),
    })
}

pub fn serialize_graph_parity_fixture(fixture: &GraphParityFixture) -> Result<String, RagasError> {
    serde_json::to_string(fixture).map_err(|error| RagasError::Parse {
        message: format!("graph parity fixture serialize failed: {error}"),
    })
}

pub fn cluster_graph_by_property(graph: &KnowledgeGraph, property_key: &str) -> Vec<GraphCluster> {
    let mut clusters = BTreeMap::<String, Vec<String>>::new();
    for node in &graph.nodes {
        if let Some(value) = node.properties.get(property_key) {
            for key in graph_property_cluster_keys(value) {
                clusters.entry(key).or_default().push(node.id.clone());
            }
        }
    }
    clusters
        .into_iter()
        .map(|(key, node_ids)| GraphCluster { key, node_ids })
        .collect()
}

/// Find up to `n` indirect node clusters — a deterministic port of Python ragas's
/// `KnowledgeGraph.find_n_indirect_clusters`. A cluster is the set of nodes on a path through the
/// graph following edges that satisfy `relationship_filter`: if `A -> B -> C` exists, `{A, B, C}` is
/// a cluster. Paths run from a start node to a leaf or up to `depth_limit` nodes long; clusters are
/// deduped, and a superset cluster evicts any of its subsets (Python's diversity rule).
///
/// `bidirectional` controls whether a matched edge is traversable in both directions. Python keys
/// this off each edge's `bidirectional` flag, which this crate's [`GraphEdge`] doesn't carry — so
/// the caller decides; symmetric-similarity clusters (the multi-hop-abstract use case) pass `true`.
///
/// **Documented divergence:** Python seeds a `random.shuffle` of the start nodes (Mersenne Twister)
/// for sampling diversity on huge graphs; RNG parity is a non-goal, so we walk start nodes in sorted
/// id order, deterministically. The set of path clusters is otherwise the same. Returns clusters as
/// node-id sets. Errors (mirroring Python's `ValueError`) if `depth_limit < 2`, `n < 1`, or no edge
/// matches the condition.
pub fn find_n_indirect_clusters(
    graph: &KnowledgeGraph,
    n: usize,
    depth_limit: usize,
    bidirectional: bool,
    relationship_filter: impl Fn(&GraphEdge) -> bool,
) -> Result<Vec<BTreeSet<String>>, RagasError> {
    if depth_limit < 2 {
        return Err(RagasError::Parse {
            message: "find_n_indirect_clusters: depth_limit must be at least 2".to_string(),
        });
    }
    if n < 1 {
        return Err(RagasError::Parse {
            message: "find_n_indirect_clusters: n must be at least 1".to_string(),
        });
    }

    // Adjacency over matched edges (plus the reverse direction when `bidirectional`).
    let mut adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut connected: BTreeSet<String> = BTreeSet::new();
    let mut unique_edges: BTreeSet<(String, String)> = BTreeSet::new();
    let mut matched = false;
    for edge in &graph.edges {
        if !relationship_filter(edge) {
            continue;
        }
        matched = true;
        adjacency
            .entry(edge.source_id.clone())
            .or_default()
            .insert(edge.target_id.clone());
        connected.insert(edge.source_id.clone());
        connected.insert(edge.target_id.clone());
        if bidirectional {
            adjacency
                .entry(edge.target_id.clone())
                .or_default()
                .insert(edge.source_id.clone());
        }
        // The undirected edge as a normalized pair (Python's `frozenset({source, target})`); a
        // self-loop collapses to `(a, a)`, matching Python's single-element frozenset for counting.
        let pair = if edge.source_id <= edge.target_id {
            (edge.source_id.clone(), edge.target_id.clone())
        } else {
            (edge.target_id.clone(), edge.source_id.clone())
        };
        unique_edges.insert(pair);
    }
    if !matched {
        return Err(RagasError::Parse {
            message: "find_n_indirect_clusters: no relationship matched the condition".to_string(),
        });
    }

    // Mirror Python's two-branch sample-size cap (bounds work on large graphs). With the RNG shuffle
    // dropped, we simply take the first `sample_size` start nodes in sorted id order.
    let sample_size = if unique_edges.len() < connected.len() {
        (n - 1) * depth_limit + 1
    } else {
        n.max(depth_limit).max(10)
    };

    let mut start_node_clusters: BTreeMap<String, BTreeSet<BTreeSet<String>>> = BTreeMap::new();
    let start_nodes: Vec<String> = adjacency.keys().take(sample_size).cloned().collect();
    for start in &start_nodes {
        let mut path = BTreeSet::new();
        dfs_indirect_clusters(
            &adjacency,
            start,
            start,
            &mut path,
            depth_limit,
            sample_size,
            &mut start_node_clusters,
        );
    }

    // Round-robin pop from each start node's clusters, deduping with superset-favoring: skip a new
    // cluster that is a subset of an existing one, and evict existing clusters it is a superset of.
    let mut buckets: Vec<BTreeSet<BTreeSet<String>>> = start_nodes
        .iter()
        .filter_map(|start| start_node_clusters.remove(start))
        .filter(|bucket| !bucket.is_empty())
        .collect();
    let mut unique: Vec<BTreeSet<String>> = Vec::new();
    let mut i = 0;
    while unique.len() < n && !buckets.is_empty() {
        let index = i % buckets.len();
        let cluster = buckets[index].pop_first().expect("bucket is non-empty");
        let is_subset = unique.iter().any(|existing| cluster.is_subset(existing));
        if !is_subset {
            unique.retain(|existing| !existing.is_subset(&cluster));
            unique.push(cluster);
        }
        if buckets[index].is_empty() {
            buckets.remove(index);
            // Don't advance `i`: removing this bucket already shifts the round-robin index.
        } else {
            i += 1;
        }
    }
    Ok(unique)
}

/// Depth-first path walk for [`find_n_indirect_clusters`]: records the current path as a cluster when
/// it reaches `depth_limit`, a leaf, or a node all of whose neighbors are already on the path.
#[allow(clippy::too_many_arguments)]
fn dfs_indirect_clusters(
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    node: &str,
    start: &str,
    path: &mut BTreeSet<String>,
    depth_limit: usize,
    sample_size: usize,
    clusters: &mut BTreeMap<String, BTreeSet<BTreeSet<String>>>,
) {
    // Stop exploring once this start node already has enough clusters (complexity guard).
    if clusters.get(start).map_or(0, BTreeSet::len) > sample_size {
        return;
    }
    path.insert(node.to_string());
    let path_length = path.len();
    let at_max_depth = path_length >= depth_limit;
    let neighbors = adjacency.get(node);
    let all_neighbors_visited = neighbors.is_none_or(|ns| ns.iter().all(|nb| path.contains(nb)));
    if path_length > 1 && (at_max_depth || all_neighbors_visited) {
        clusters
            .entry(start.to_string())
            .or_default()
            .insert(path.clone());
    } else if let Some(ns) = neighbors {
        for neighbor in ns {
            if !path.contains(neighbor) {
                dfs_indirect_clusters(
                    adjacency,
                    neighbor,
                    start,
                    path,
                    depth_limit,
                    sample_size,
                    clusters,
                );
            }
        }
    }
    path.remove(node);
}

pub fn query_graph_advanced(graph: &KnowledgeGraph, query: &GraphAdvancedQuery) -> Vec<GraphNode> {
    graph
        .nodes
        .iter()
        .filter(|node| {
            query
                .node_type
                .as_ref()
                .is_none_or(|node_type| node.node_type == *node_type)
        })
        .filter(
            |node| match (query.property_key.as_deref(), query.property_value.as_ref()) {
                (Some(key), Some(expected)) => node
                    .properties
                    .get(key)
                    .is_some_and(|actual| graph_property_matches(actual, expected)),
                _ => true,
            },
        )
        .filter(|node| {
            query
                .outgoing_relationship
                .as_ref()
                .is_none_or(|relationship| {
                    graph
                        .edges
                        .iter()
                        .any(|edge| edge.source_id == node.id && edge.relationship == *relationship)
                })
        })
        .cloned()
        .collect()
}

pub fn normalize_extraction_properties(
    extractions: ExtractionBundle,
) -> BTreeMap<String, GraphProperty> {
    let mut properties = BTreeMap::new();
    properties.insert(
        "entities".to_string(),
        GraphProperty::TextList(normalize_text_list(extractions.entities)),
    );
    properties.insert(
        "themes".to_string(),
        GraphProperty::TextList(normalize_text_list(extractions.themes)),
    );
    properties.insert(
        "summary".to_string(),
        GraphProperty::Text(extractions.summary.trim().to_string()),
    );
    properties
}

pub fn parse_llm_extractor_output(input: &str) -> Result<ExtractionBundle, RagasError> {
    serde_json::from_str(input).map_err(|error| RagasError::Parse {
        message: format!("llm extractor output parse failed: {error}"),
    })
}

pub fn filter_graph_by_property(
    graph: &KnowledgeGraph,
    property_key: &str,
    expected: &GraphProperty,
) -> KnowledgeGraph {
    let kept_ids = graph
        .nodes
        .iter()
        .filter(|node| {
            node.properties
                .get(property_key)
                .is_some_and(|actual| graph_property_matches(actual, expected))
        })
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();

    KnowledgeGraph {
        nodes: graph
            .nodes
            .iter()
            .filter(|node| kept_ids.contains(&node.id))
            .cloned()
            .collect(),
        edges: graph
            .edges
            .iter()
            .filter(|edge| kept_ids.contains(&edge.source_id) && kept_ids.contains(&edge.target_id))
            .cloned()
            .collect(),
    }
}

pub fn render_synthesizer_prompt_snapshot(
    snapshot: &SynthesizerPromptSnapshot,
) -> Result<RenderedSynthesizerPromptSnapshot, RagasError> {
    let mut rendered_messages = Vec::with_capacity(snapshot.messages.len());
    for message in &snapshot.messages {
        let mut content = message.template.replace(
            "{{strategy}}",
            synthesizer_strategy_label(snapshot.strategy),
        );
        for (name, value) in &snapshot.variables {
            content = content.replace(&format!("{{{{{name}}}}}"), value);
        }
        if content.contains("{{") || content.contains("}}") {
            return Err(RagasError::Prompt {
                message: format!(
                    "unresolved synthesizer prompt variable in role '{}'",
                    message.role
                ),
            });
        }
        rendered_messages.push(RenderedSynthesizerPromptMessage {
            role: message.role.clone(),
            content,
        });
    }

    Ok(RenderedSynthesizerPromptSnapshot {
        strategy: snapshot.strategy,
        variables: snapshot.variables.clone(),
        rendered_messages,
    })
}

pub fn compare_synthesized_sample_fixture(
    expected: &SynthesizedSample,
    actual: &SynthesizedSample,
) -> SynthesizerSampleComparison {
    if expected == actual {
        return SynthesizerSampleComparison {
            matches: true,
            drift: None,
        };
    }

    let mut drift = Vec::new();
    if expected.hop_count != actual.hop_count {
        drift.push("hop_count");
    }
    if expected.source_node_ids != actual.source_node_ids {
        drift.push("source_node_ids");
    }
    if expected.persona != actual.persona {
        drift.push("persona");
    }
    if expected.sample != actual.sample {
        drift.push("sample");
    }

    SynthesizerSampleComparison {
        matches: false,
        drift: Some(if drift.is_empty() {
            "unknown synthesized sample drift".to_string()
        } else {
            format!("synthesized sample drift in {}", drift.join(", "))
        }),
    }
}

pub fn synthesize_pre_chunked_samples(
    chunks: &[TextChunk],
    persona: &Persona,
) -> Vec<SynthesizedSample> {
    chunks
        .iter()
        .map(|chunk| {
            let sample = SingleTurnSample::new(
                format!(
                    "{} should verify the provided pre-chunked evidence from {}.",
                    persona.role, chunk.id
                ),
                chunk.text.clone(),
                vec![chunk.text.clone()],
            )
            .with_reference(chunk.text.clone())
            .with_metadata("synthesis_type", "pre-chunked")
            .with_metadata("source_chunk_id", chunk.id.clone())
            .with_metadata("source_id", chunk.source_id.clone())
            .with_metadata("persona", persona.name.clone());
            SynthesizedSample {
                sample,
                persona: persona.clone(),
                source_node_ids: vec![chunk.id.clone()],
                hop_count: 1,
            }
        })
        .collect()
}

fn graph_property_cluster_keys(value: &GraphProperty) -> Vec<String> {
    match value {
        GraphProperty::Text(value) => vec![value.clone()],
        GraphProperty::Number(value) => vec![format!("{value:.6}")],
        GraphProperty::Boolean(value) => vec![value.to_string()],
        GraphProperty::TextList(values) => values.clone(),
        // Embedding vectors are not sensible discrete cluster keys.
        GraphProperty::Vector(_) => Vec::new(),
    }
}

fn graph_property_matches(actual: &GraphProperty, expected: &GraphProperty) -> bool {
    if actual == expected {
        return true;
    }
    match (actual, expected) {
        (GraphProperty::TextList(values), GraphProperty::Text(expected)) => {
            values.iter().any(|value| value == expected)
        }
        (GraphProperty::Text(value), GraphProperty::TextList(expected_values)) => {
            expected_values.iter().any(|expected| expected == value)
        }
        _ => false,
    }
}

fn normalize_text_list(mut values: Vec<String>) -> Vec<String> {
    values.iter_mut().for_each(|value| {
        let trimmed = value.trim().to_string();
        *value = trimmed;
    });
    values.retain(|value| !value.is_empty());
    values.sort();
    values.dedup();
    values
}

fn synthesizer_strategy_label(strategy: SynthesizerStrategy) -> &'static str {
    match strategy {
        SynthesizerStrategy::SingleHop => "single-hop",
        SynthesizerStrategy::MultiHop => "multi-hop",
        SynthesizerStrategy::PreChunked => "pre-chunked",
    }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(mut self, node: GraphNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn add_edge(mut self, edge: GraphEdge) -> Self {
        self.edges.push(edge);
        self
    }

    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn nodes_by_type(&self, node_type: &str) -> Vec<&GraphNode> {
        self.nodes
            .iter()
            .filter(|node| node.node_type == node_type)
            .collect()
    }

    pub fn edges_by_relationship(&self, relationship: &str) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.relationship == relationship)
            .collect()
    }

    pub fn nodes_with_property(
        &self,
        key: &str,
        expected: Option<&GraphProperty>,
    ) -> Vec<&GraphNode> {
        self.nodes
            .iter()
            .filter(|node| {
                node.properties
                    .get(key)
                    .is_some_and(|value| expected.is_none_or(|expected| value == expected))
            })
            .collect()
    }

    pub fn neighbors(&self, source_id: &str, relationship: &str) -> Vec<&GraphNode> {
        self.edges
            .iter()
            .filter(|edge| edge.source_id == source_id && edge.relationship == relationship)
            .filter_map(|edge| self.node(&edge.target_id))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextChunk {
    pub id: String,
    pub source_id: String,
    pub index: usize,
    pub text: String,
    pub metadata: BTreeMap<String, String>,
}

impl TextChunk {
    pub fn new(
        id: impl Into<String>,
        source_id: impl Into<String>,
        index: usize,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source_id: source_id.into(),
            index,
            text: text.into(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn to_graph_node(&self) -> GraphNode {
        GraphNode::new(self.id.clone(), "chunk")
            .with_property("text", GraphProperty::Text(self.text.clone()))
            .with_property("source_id", GraphProperty::Text(self.source_id.clone()))
            .with_property("chunk_index", GraphProperty::Number(self.index as f64))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionBundle {
    pub entities: Vec<String>,
    pub themes: Vec<String>,
    pub summary: String,
}

impl ExtractionBundle {
    pub fn new(entities: Vec<String>, themes: Vec<String>, summary: impl Into<String>) -> Self {
        Self {
            entities,
            themes,
            summary: summary.into(),
        }
    }
}

pub fn split_text_into_chunks(source_id: &str, text: &str, max_chars: usize) -> Vec<TextChunk> {
    let max_chars = max_chars.max(1);
    let mut chunks = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let next_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };

        if !current.is_empty() && next_len > max_chars {
            chunks.push(make_chunk(source_id, chunks.len(), current));
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        chunks.push(make_chunk(source_id, chunks.len(), current));
    }

    chunks
}

pub fn attach_extractions(
    mut graph: KnowledgeGraph,
    node_id: &str,
    extractions: ExtractionBundle,
) -> KnowledgeGraph {
    if let Some(node) = graph.nodes.iter_mut().find(|node| node.id == node_id) {
        node.properties
            .extend(normalize_extraction_properties(extractions));
    }

    graph
}

pub fn build_chunk_relationships(
    mut graph: KnowledgeGraph,
    source_id: &str,
    chunks: &[TextChunk],
) -> KnowledgeGraph {
    for chunk in chunks {
        graph = graph.add_edge(
            GraphEdge::new(source_id, chunk.id.clone(), "contains")
                .with_property("order", GraphProperty::Number(chunk.index as f64)),
        );
    }

    for window in chunks.windows(2) {
        let source = &window[0];
        let target = &window[1];
        graph = graph.add_edge(
            GraphEdge::new(source.id.clone(), target.id.clone(), "next")
                .with_property("order", GraphProperty::Number(source.index as f64)),
        );
    }

    graph
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub role: String,
    pub goals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonaGenerator {
    pub seed: String,
}

impl PersonaGenerator {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }

    pub fn generate(
        &self,
        name: impl Into<String>,
        role: impl Into<String>,
        goals: Vec<String>,
    ) -> Persona {
        Persona {
            name: name.into(),
            role: role.into(),
            goals,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SynthesizedSample {
    pub sample: SingleTurnSample,
    pub persona: Persona,
    pub source_node_ids: Vec<String>,
    pub hop_count: usize,
}

pub fn synthesize_single_hop_sample(
    graph: &KnowledgeGraph,
    chunk_id: &str,
    persona: &Persona,
) -> Option<SynthesizedSample> {
    let node = graph.node(chunk_id)?;
    let context = text_property(node, "text")?.to_string();
    let summary = text_property(node, "summary")
        .unwrap_or(&context)
        .to_string();
    let goal = persona
        .goals
        .first()
        .map(String::as_str)
        .unwrap_or("evaluate");
    let sample = SingleTurnSample::new(
        format!(
            "As a {}, ask a grounded single-hop question to {} using {}.",
            persona.role, goal, chunk_id
        ),
        summary.clone(),
        vec![context],
    )
    .with_reference(summary)
    .with_metadata("synthesis_type", "single-hop")
    .with_metadata("persona", persona.name.clone())
    .with_metadata("source_node_ids", chunk_id.to_string());

    Some(SynthesizedSample {
        sample,
        persona: persona.clone(),
        source_node_ids: vec![chunk_id.to_string()],
        hop_count: 1,
    })
}

pub fn synthesize_multi_hop_sample(
    graph: &KnowledgeGraph,
    start_node_id: &str,
    relationship: &str,
    persona: &Persona,
) -> Option<SynthesizedSample> {
    let start = graph.node(start_node_id)?;
    let mut nodes = vec![start];
    nodes.extend(graph.neighbors(start_node_id, relationship));
    if nodes.len() < 2 {
        return None;
    }

    let mut source_node_ids = Vec::with_capacity(nodes.len());
    let mut contexts = Vec::with_capacity(nodes.len());
    let mut summaries = Vec::with_capacity(nodes.len());
    for node in nodes {
        source_node_ids.push(node.id.clone());
        contexts.push(text_property(node, "text")?.to_string());
        summaries.push(
            text_property(node, "summary")
                .unwrap_or(&contexts[contexts.len() - 1])
                .to_string(),
        );
    }

    let goal = persona
        .goals
        .first()
        .map(String::as_str)
        .unwrap_or("compare");
    let response = summaries.join(" ");
    let sample = SingleTurnSample::new(
        format!(
            "As a {}, ask a multi-hop question to {} across relationship {}.",
            persona.role, goal, relationship
        ),
        response.clone(),
        contexts,
    )
    .with_reference(response)
    .with_metadata("synthesis_type", "multi-hop")
    .with_metadata("persona", persona.name.clone())
    .with_metadata("relationship", relationship.to_string())
    .with_metadata("source_node_ids", source_node_ids.join(","));

    Some(SynthesizedSample {
        sample,
        persona: persona.clone(),
        hop_count: source_node_ids.len(),
        source_node_ids,
    })
}

fn make_chunk(source_id: &str, index: usize, text: String) -> TextChunk {
    let mut chunk = TextChunk::new(
        format!("{source_id}-chunk-{index}"),
        source_id.to_string(),
        index,
        text,
    );
    chunk
        .metadata
        .insert("source_id".to_string(), source_id.to_string());
    chunk
        .metadata
        .insert("chunk_index".to_string(), index.to_string());
    chunk
}

fn text_property<'a>(node: &'a GraphNode, key: &str) -> Option<&'a str> {
    match node.properties.get(key) {
        Some(GraphProperty::Text(value)) => Some(value.as_str()),
        _ => None,
    }
}

/// One LLM-synthesized question/answer pair, parsed (with repair) from a `generate` call.
#[derive(Debug, Deserialize)]
struct SynthesizedQa {
    question: String,
    answer: String,
}

/// Deserialize a `{"question": "...", "answer": "..."}` object from an LLM response,
/// tolerating markdown fences or surrounding prose by extracting the outermost `{ .. }`
/// block (the JSON-repair path, mirroring `metric::parse_json`/`extract_json_block`).
fn parse_synthesized_qa(content: &str, context: &str) -> Result<SynthesizedQa, RagasError> {
    let block = match (content.find('{'), content.rfind('}')) {
        (Some(start), Some(end)) if end >= start => &content[start..=end],
        _ => content.trim(),
    };
    let qa: SynthesizedQa = serde_json::from_str(block).map_err(|error| RagasError::Parse {
        message: format!("{context}: {error}"),
    })?;
    if qa.question.trim().is_empty() || qa.answer.trim().is_empty() {
        return Err(RagasError::Parse {
            message: format!("{context}: empty question or answer"),
        });
    }
    Ok(qa)
}

/// Real, runnable test-set generation pipeline.
///
/// Given raw document text and an [`LlmProvider`], it builds the [`KnowledgeGraph`]
/// (chunk → graph-node → `contains`/`next` edges, all reusing the existing transform
/// code in this module) and then drives the LLM to *synthesize* question/answer pairs
/// grounded in chunk content, returning a crate [`EvaluationDataset`].
///
/// Every generated sample comes from a real [`LlmProvider::generate`] call — nothing is
/// hardcoded or pure-template. The chunk text is the retrieved context, the LLM question
/// is `user_input`, and the LLM answer is `response`/`reference`.
///
/// NON-GOAL: byte/score parity with Python ragas' `np.random` / global-MT scenario and
/// node selection. We do NOT attempt to reproduce that RNG. Instead, selection order is
/// the deterministic document order of the knowledge graph (chunk nodes in the order they
/// were split, then adjacent `next` pairs for multi-hop). This is an explicitly captured,
/// reproducible order rather than a port of ragas' random sampling.
pub struct Synthesizer {
    llm: Arc<dyn LlmProvider>,
    chunk_max_chars: usize,
    multi_hop: bool,
}

impl Synthesizer {
    /// Create a synthesizer over the given LLM provider. Defaults: ~1000-char chunks,
    /// multi-hop enabled.
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            llm,
            chunk_max_chars: 1000,
            multi_hop: true,
        }
    }

    pub fn with_chunk_max_chars(mut self, chunk_max_chars: usize) -> Self {
        self.chunk_max_chars = chunk_max_chars.max(1);
        self
    }

    pub fn with_multi_hop(mut self, multi_hop: bool) -> Self {
        self.multi_hop = multi_hop;
        self
    }

    /// Build a knowledge graph from `document_text` and synthesize an
    /// [`EvaluationDataset`] from it using the LLM.
    ///
    /// Returns `Err(RagasError::EmptyDataset)` when the document has no usable text, and
    /// propagates `Err` from the provider or from malformed LLM JSON.
    pub async fn generate_testset(
        &self,
        source_id: &str,
        document_text: &str,
    ) -> Result<EvaluationDataset, RagasError> {
        let chunks = split_text_into_chunks(source_id, document_text, self.chunk_max_chars);
        if chunks.is_empty() {
            return Err(RagasError::EmptyDataset);
        }

        // Reuse the real transform/relationship code: each chunk becomes a graph node and
        // `contains`/`next` edges are built between them.
        let graph = chunks.iter().fold(
            KnowledgeGraph::new().add_node(GraphNode::new(source_id, "document")),
            |graph, chunk| graph.add_node(chunk.to_graph_node()),
        );
        let graph = build_chunk_relationships(graph, source_id, &chunks);

        let mut samples = Vec::new();

        // Single-hop: deterministic document-order traversal of chunk nodes.
        for chunk in &graph.nodes_by_type("chunk") {
            let context = match text_property(chunk, "text") {
                Some(text) if !text.trim().is_empty() => text.to_string(),
                _ => continue,
            };
            let qa = self.generate_single_hop_qa(&context).await?;
            samples.push(
                SingleTurnSample::new(qa.question, qa.answer.clone(), vec![context])
                    .with_reference(qa.answer)
                    .with_metadata("synthesis_type", "single-hop")
                    .with_metadata("source_node_ids", chunk.id.clone()),
            );
        }

        // Multi-hop: adjacent `next`-linked chunk pairs, again in deterministic graph order.
        if self.multi_hop {
            for edge in graph.edges_by_relationship("next") {
                let (Some(source), Some(target)) =
                    (graph.node(&edge.source_id), graph.node(&edge.target_id))
                else {
                    continue;
                };
                let (Some(source_text), Some(target_text)) =
                    (text_property(source, "text"), text_property(target, "text"))
                else {
                    continue;
                };
                if source_text.trim().is_empty() || target_text.trim().is_empty() {
                    continue;
                }
                let contexts = vec![source_text.to_string(), target_text.to_string()];
                let qa = self.generate_multi_hop_qa(source_text, target_text).await?;
                samples.push(
                    SingleTurnSample::new(qa.question, qa.answer.clone(), contexts)
                        .with_reference(qa.answer)
                        .with_metadata("synthesis_type", "multi-hop")
                        .with_metadata(
                            "source_node_ids",
                            format!("{},{}", edge.source_id, edge.target_id),
                        ),
                );
            }
        }

        EvaluationDataset::new(samples)
    }

    /// Ask the LLM to write a grounded single-hop Q/A pair answerable from one chunk.
    async fn generate_single_hop_qa(&self, context: &str) -> Result<SynthesizedQa, RagasError> {
        let prompt = format!(
            "You generate evaluation data for a retrieval system. Read the CONTEXT below \
and write ONE self-contained question that is fully answerable using ONLY that context, \
together with its correct answer. Do not ask about anything not present in the context. \
Return ONLY JSON of the form {{\"question\": \"...\", \"answer\": \"...\"}}.\n\nCONTEXT:\n{context}"
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        parse_synthesized_qa(&response.content, "single-hop testset synthesis")
    }

    /// Ask the LLM to write a multi-hop Q/A pair that requires combining two chunks.
    async fn generate_multi_hop_qa(
        &self,
        first_context: &str,
        second_context: &str,
    ) -> Result<SynthesizedQa, RagasError> {
        let prompt = format!(
            "You generate evaluation data for a retrieval system. Read the TWO context \
passages below and write ONE question whose answer requires combining information from \
BOTH passages, together with its correct answer. Use ONLY information present in the \
passages. Return ONLY JSON of the form {{\"question\": \"...\", \"answer\": \"...\"}}.\n\n\
CONTEXT A:\n{first_context}\n\nCONTEXT B:\n{second_context}"
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        parse_synthesized_qa(&response.content, "multi-hop testset synthesis")
    }
}

/// Convenience free function mirroring [`Synthesizer::generate_testset`] for callers that
/// just have a document and a provider.
pub async fn generate_testset(
    llm: Arc<dyn LlmProvider>,
    source_id: &str,
    document_text: &str,
) -> Result<EvaluationDataset, RagasError> {
    Synthesizer::new(llm)
        .generate_testset(source_id, document_text)
        .await
}

/// The kind of property an [`LlmExtractor`] pulls out of a graph node's text.
///
/// Faithful port of the seven Python `ragas.testset.transforms.extractors.llm_based`
/// `LLMBasedExtractor` subclasses. Each kind differs only by its instruction, the JSON shape
/// it asks the model for (a single string vs a string list), and the graph property it writes
/// — the orchestration (read node text → chunk → `generate` → JSON-repair parse) is shared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmExtractorKind {
    /// `summary` — a concise summary of the text (single value). Python `SummaryExtractor`.
    Summary,
    /// `keyphrases` — top keyphrases (list). Python `KeyphrasesExtractor` (default max 5).
    Keyphrases,
    /// `title` — the document title (single value). Python `TitleExtractor`.
    Title,
    /// `headlines` — section headlines (list). Python `HeadlinesExtractor` (default max 5).
    Headlines,
    /// `entities` — named entities (list). Python `NERExtractor` (default max 10).
    Ner,
    /// `themes` — main themes/concepts (list). Python `ThemesExtractor` (default max 10).
    Themes,
    /// `topic_description` — a concise topic description (single value). Python
    /// `TopicDescriptionExtractor`.
    TopicDescription,
}

impl LlmExtractorKind {
    /// The graph property name this kind writes (mirrors the Python `property_name`).
    pub fn property_name(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Keyphrases => "keyphrases",
            Self::Title => "title",
            Self::Headlines => "headlines",
            Self::Ner => "entities",
            Self::Themes => "themes",
            Self::TopicDescription => "topic_description",
        }
    }

    /// The JSON object key the model is asked to emit (mirrors the Python pydantic output
    /// model field).
    fn json_key(self) -> &'static str {
        match self {
            Self::Summary | Self::Title => "text",
            Self::Keyphrases => "keyphrases",
            Self::Headlines => "headlines",
            Self::Ner => "entities",
            Self::Themes => "output",
            Self::TopicDescription => "description",
        }
    }

    /// Whether the output is a list of strings (vs a single string).
    fn is_list(self) -> bool {
        matches!(
            self,
            Self::Keyphrases | Self::Headlines | Self::Ner | Self::Themes
        )
    }

    /// Default per-chunk item cap for the list kinds (ignored by single-value kinds),
    /// matching the Python extractor defaults.
    fn default_max_num(self) -> usize {
        match self {
            Self::Keyphrases | Self::Headlines => 5,
            Self::Ner | Self::Themes => 10,
            _ => 0,
        }
    }

    /// The extraction instruction, mirroring the Python prompt `instruction` strings.
    fn instruction(self, max_num: usize) -> String {
        match self {
            Self::Summary => "Summarize the given text in less than 10 sentences.".to_string(),
            Self::Keyphrases => format!("Extract top {max_num} keyphrases from the given text."),
            Self::Title => "Extract the title of the given document.".to_string(),
            Self::Headlines => format!(
                "Extract the most important {max_num} headlines from the given text that can be \
used to split the text into independent sections. Focus on Level 2 and Level 3 headings."
            ),
            Self::Ner => format!(
                "Extract the named entities from the given text, limiting the output to the top \
entities. Ensure the number of entities does not exceed {max_num}."
            ),
            Self::Themes => "Extract the main themes and concepts from the given text.".to_string(),
            Self::TopicDescription => {
                "Provide a concise description of the main topic(s) discussed in the following text."
                    .to_string()
            }
        }
    }
}

/// LLM-backed extractor that writes a property onto a knowledge-graph node from its text.
///
/// This is the runnable analog of Python `ragas`'s `LLMBasedExtractor` family: it reads the
/// node's `text` property, splits it into chunks (a char-based substitute for ragas's tiktoken
/// `split_text_by_token_limit` — token parity is an explicit non-goal), prompts a real
/// [`LlmProvider`] for the requested property as JSON, and parses the response with the same
/// outermost-`{ .. }` JSON-repair path used by the synthesizer.
///
/// Single-value kinds (Summary/Title/TopicDescription) use only the first chunk (matching
/// Python's `chunks[0]`); list kinds (Keyphrases/Headlines/NER/Themes) call the model per chunk
/// and concatenate the results, matching Python's `extend` loop (no post-dedup or truncation —
/// `max_num` only bounds the prompt, as in Python).
pub struct LlmExtractor {
    llm: Arc<dyn LlmProvider>,
    kind: LlmExtractorKind,
    max_num: usize,
    max_chars: usize,
}

impl LlmExtractor {
    /// A char-budget substitute for Python's 32000-*token* `max_token_limit`. Chars are not
    /// tokens (token parity is a non-goal); in practice node texts are well under this, so a
    /// single chunk is used.
    const DEFAULT_MAX_CHARS: usize = 32_000;

    /// Create an extractor of the given kind over `llm`, with the Python default `max_num`.
    pub fn new(llm: Arc<dyn LlmProvider>, kind: LlmExtractorKind) -> Self {
        Self {
            llm,
            kind,
            max_num: kind.default_max_num(),
            max_chars: Self::DEFAULT_MAX_CHARS,
        }
    }

    /// Override the per-chunk item cap for list kinds (no effect on single-value kinds).
    pub fn with_max_num(mut self, max_num: usize) -> Self {
        self.max_num = max_num;
        self
    }

    /// Override the per-chunk char budget used to split long node text.
    pub fn with_max_chars(mut self, max_chars: usize) -> Self {
        self.max_chars = max_chars.max(1);
        self
    }

    /// The property name this extractor writes.
    pub fn property_name(&self) -> &'static str {
        self.kind.property_name()
    }

    /// Extract this extractor's property from `node`'s `text`, returning `(property_name,
    /// value)` ready to insert into the node's properties — the same shape as Python's
    /// `extract(node) -> (property_name, value)`.
    ///
    /// When the node has no non-empty `text` property, returns an empty value (an empty list
    /// for list kinds, an empty string for single-value kinds) without calling the model,
    /// mirroring Python's `return self.property_name, None/[]` leniency rather than erroring.
    pub async fn extract(&self, node: &GraphNode) -> Result<(String, GraphProperty), RagasError> {
        let name = self.kind.property_name().to_string();
        let text = text_property(node, "text").unwrap_or("").trim();
        if text.is_empty() {
            let empty = if self.kind.is_list() {
                GraphProperty::TextList(Vec::new())
            } else {
                GraphProperty::Text(String::new())
            };
            return Ok((name, empty));
        }

        let chunks = split_text_into_chunks(&node.id, text, self.max_chars);

        if self.kind.is_list() {
            let mut items = Vec::new();
            for chunk in &chunks {
                items.extend(self.extract_list_chunk(&chunk.text).await?);
            }
            Ok((name, GraphProperty::TextList(items)))
        } else {
            // Single-value kinds use only the first chunk, matching Python's `chunks[0]`.
            let first = chunks
                .first()
                .map(|chunk| chunk.text.as_str())
                .unwrap_or(text);
            let value = self.extract_single_chunk(first).await?;
            Ok((name, GraphProperty::Text(value)))
        }
    }

    async fn extract_list_chunk(&self, chunk: &str) -> Result<Vec<String>, RagasError> {
        let context = self.context();
        let block = self.generate_block(chunk).await?;
        let value = parse_json_block(&block, &context)?;
        let items = value
            .get(self.kind.json_key())
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| RagasError::Parse {
                message: format!("{context}: missing list field '{}'", self.kind.json_key()),
            })?
            .iter()
            .filter_map(|item| item.as_str().map(|item| item.trim().to_string()))
            .filter(|item| !item.is_empty())
            .collect();
        Ok(items)
    }

    async fn extract_single_chunk(&self, chunk: &str) -> Result<String, RagasError> {
        let context = self.context();
        let block = self.generate_block(chunk).await?;
        let value = parse_json_block(&block, &context)?;
        let text = value
            .get(self.kind.json_key())
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| RagasError::Parse {
                message: format!("{context}: missing string field '{}'", self.kind.json_key()),
            })?;
        Ok(text.trim().to_string())
    }

    async fn generate_block(&self, chunk: &str) -> Result<String, RagasError> {
        let instruction = self.kind.instruction(self.max_num);
        let key = self.kind.json_key();
        let shape = if self.kind.is_list() {
            format!("{{\"{key}\": [\"...\", \"...\"]}}")
        } else {
            format!("{{\"{key}\": \"...\"}}")
        };
        let prompt =
            format!("{instruction}\nReturn ONLY JSON of the form {shape}.\n\nTEXT:\n{chunk}");
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        Ok(response.content)
    }

    fn context(&self) -> String {
        format!("llm extractor '{}'", self.kind.property_name())
    }
}

/// Extract the outermost `{ .. }` block from an LLM response (tolerating prose / markdown
/// fences) and parse it as JSON — the JSON-repair path shared with [`parse_synthesized_qa`].
fn parse_json_block(content: &str, context: &str) -> Result<serde_json::Value, RagasError> {
    let block = match (content.find('{'), content.rfind('}')) {
        (Some(start), Some(end)) if end >= start => &content[start..=end],
        _ => content.trim(),
    };
    serde_json::from_str(block).map_err(|error| RagasError::Parse {
        message: format!("{context}: {error}"),
    })
}

/// Drive a real [`LlmProvider`] to extract the `{entities, themes, summary}` triple from a
/// node's text and assemble it into an [`ExtractionBundle`] — the input expected by
/// [`attach_extractions`]. This wires the previously hand-fed extraction substrate to a live
/// model. The three extractors run in a **fixed sequential order** (NER → Themes → Summary) so
/// a scripted/mock provider is deterministic.
pub async fn extract_bundle(
    llm: Arc<dyn LlmProvider>,
    node: &GraphNode,
) -> Result<ExtractionBundle, RagasError> {
    let entities = list_property(
        LlmExtractor::new(llm.clone(), LlmExtractorKind::Ner)
            .extract(node)
            .await?,
    );
    let themes = list_property(
        LlmExtractor::new(llm.clone(), LlmExtractorKind::Themes)
            .extract(node)
            .await?,
    );
    let summary = text_value(
        LlmExtractor::new(llm, LlmExtractorKind::Summary)
            .extract(node)
            .await?,
    );
    Ok(ExtractionBundle::new(entities, themes, summary))
}

/// Pull the list value out of an [`LlmExtractor::extract`] result. Defensive fallback only:
/// the list kinds always return [`GraphProperty::TextList`], so the empty branch is
/// unreachable for the kinds [`extract_bundle`] passes here.
fn list_property((_, property): (String, GraphProperty)) -> Vec<String> {
    match property {
        GraphProperty::TextList(values) => values,
        _ => Vec::new(),
    }
}

/// Pull the single text value out of an [`LlmExtractor::extract`] result. Defensive fallback
/// only: the single-value kinds always return [`GraphProperty::Text`].
fn text_value((_, property): (String, GraphProperty)) -> String {
    match property {
        GraphProperty::Text(value) => value,
        _ => String::new(),
    }
}

/// Embedding-backed extractor that writes a dense vector onto a graph node from its text.
///
/// The runnable analog of Python `ragas`'s `EmbeddingExtractor`: it reads the node's text
/// property, embeds it via a real [`EmbeddingProvider`], and returns a [`GraphProperty::Vector`]
/// under `property_name` (default `"embedding"`). Unlike the lenient [`LlmExtractor`], a node
/// whose embed-text property is missing or non-text is an **error** (mirroring Python's
/// `ValueError`), since an embedding has no meaningful empty value.
pub struct EmbeddingExtractor {
    embedding: Arc<dyn EmbeddingProvider>,
    property_name: String,
    embed_property_name: String,
}

impl EmbeddingExtractor {
    /// Create an extractor over `embedding`, writing `"embedding"` from the node's `"text"`
    /// property (this module's text key; Python's default is `page_content`).
    pub fn new(embedding: Arc<dyn EmbeddingProvider>) -> Self {
        Self {
            embedding,
            property_name: "embedding".to_string(),
            embed_property_name: "text".to_string(),
        }
    }

    /// Override the property the embedding is written to (Python `property_name`).
    pub fn with_property_name(mut self, property_name: impl Into<String>) -> Self {
        self.property_name = property_name.into();
        self
    }

    /// Override the property the text to embed is read from (Python `embed_property_name`).
    pub fn with_embed_property_name(mut self, embed_property_name: impl Into<String>) -> Self {
        self.embed_property_name = embed_property_name.into();
        self
    }

    /// Embed `node`'s text and return `(property_name, GraphProperty::Vector)`.
    pub async fn extract(&self, node: &GraphNode) -> Result<(String, GraphProperty), RagasError> {
        let Some(text) = text_property(node, &self.embed_property_name) else {
            return Err(RagasError::Parse {
                message: format!(
                    "embedding extractor: node '{}' has no text property '{}'",
                    node.id, self.embed_property_name
                ),
            });
        };
        let mut response = self
            .embedding
            .embed(EmbeddingRequest {
                input: vec![text.to_string()],
            })
            .await?;
        if response.embeddings.len() != 1 {
            return Err(RagasError::Provider {
                message: format!(
                    "embedding extractor: expected 1 embedding, got {}",
                    response.embeddings.len()
                ),
            });
        }
        let vector = response.embeddings.remove(0);
        Ok((self.property_name.clone(), GraphProperty::Vector(vector)))
    }
}

/// Add `cosine_similarity` relationships between graph nodes that carry an `embedding`
/// [`GraphProperty::Vector`], for every pair whose cosine similarity is `>= threshold`.
///
/// Faithful to Python `ragas`'s `CosineSimilarityBuilder` (property `"embedding"`, relationship
/// `"cosine_similarity"`, score carried as a `cosine_similarity` edge property), with one
/// **documented divergence**: Python errors if *any* node lacks the embedding because its
/// transforms engine pre-filters; that engine doesn't exist here yet, so this filters to the
/// embedded nodes instead of erroring. Embedded nodes must share one dimension (mirrors
/// `_validate_embedding_shapes`); a mismatch is an error. One directed edge is added per
/// `i < j` pair (the relationship is undirected — treat source/target symmetrically).
pub fn build_cosine_relationships(
    mut graph: KnowledgeGraph,
    threshold: f64,
) -> Result<KnowledgeGraph, RagasError> {
    let embedded: Vec<(usize, Vec<f32>)> = graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(idx, node)| match node.properties.get("embedding") {
            Some(GraphProperty::Vector(vector)) => Some((idx, vector.clone())),
            _ => None,
        })
        .collect();

    if let Some((_, first)) = embedded.first() {
        let dimension = first.len();
        if let Some((idx, vector)) = embedded.iter().find(|(_, v)| v.len() != dimension) {
            return Err(RagasError::Parse {
                message: format!(
                    "cosine builder: embedding on node '{}' has length {} (expected {dimension})",
                    graph.nodes[*idx].id,
                    vector.len()
                ),
            });
        }
    }

    let mut new_edges = Vec::new();
    for a in 0..embedded.len() {
        for b in (a + 1)..embedded.len() {
            let (i, vi) = (embedded[a].0, &embedded[a].1);
            let (j, vj) = (embedded[b].0, &embedded[b].1);
            let score = cosine_similarity(vi, vj);
            if score >= threshold {
                new_edges.push(
                    GraphEdge::new(
                        graph.nodes[i].id.clone(),
                        graph.nodes[j].id.clone(),
                        "cosine_similarity",
                    )
                    .with_property("cosine_similarity", GraphProperty::Number(score)),
                );
            }
        }
    }
    for edge in new_edges {
        graph = graph.add_edge(edge);
    }
    Ok(graph)
}

/// Add `entities_overlap` relationships between graph nodes whose `entities` lists share
/// fuzzily-matching items — a faithful port of Python ragas's `OverlapScoreBuilder` (the
/// entity-overlap builder used by the default testset pipeline).
///
/// For each directed pair `i < j`, every non-noisy entity of node `i` is compared to every
/// non-noisy entity of node `j` via case-insensitive Jaro-Winkler similarity (`1 - distance`,
/// reusing the Phase-2 [`crate::string_distance_similarity_with`]); a comparison counts as a
/// match when similarity `>= distance_threshold`. The overlap score is `matches / comparisons`
/// and an `entities_overlap` edge (carrying `entities_overlap_score` and the matched
/// `overlapped_items`) is added when that score `>= threshold`. "Noisy" entities — the top
/// ~5% most frequent across all nodes (at least one) — are excluded, mirroring Python's
/// `_get_noisy_items`.
///
/// Python defaults: `distance_threshold = 0.9`, `threshold = 0.01`. Like
/// [`build_cosine_relationships`], this filters to nodes carrying an `entities`
/// [`GraphProperty::TextList`] instead of erroring on a node that lacks it (documented
/// divergence — the transforms-engine pre-filter doesn't exist yet). Edges are **directed**
/// (Python's overlap relationship is not bidirectional, unlike its cosine/Jaccard ones).
/// `overlapped_items` is a [`GraphProperty::TextList`] of `"x => y"` strings (Python stores
/// `(x, y)` tuples; `GraphProperty` has no tuple type — a representation, not a logic, change).
pub fn build_overlap_relationships(
    mut graph: KnowledgeGraph,
    distance_threshold: f64,
    threshold: f64,
) -> KnowledgeGraph {
    let entitied: Vec<(usize, Vec<String>)> = graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(idx, node)| match node.properties.get("entities") {
            Some(GraphProperty::TextList(items)) => Some((idx, items.clone())),
            _ => None,
        })
        .collect();

    let noisy = noisy_entities(&entitied);

    let mut new_edges = Vec::new();
    for a in 0..entitied.len() {
        for b in (a + 1)..entitied.len() {
            let (i, items_i) = (entitied[a].0, &entitied[a].1);
            let (j, items_j) = (entitied[b].0, &entitied[b].1);

            let mut comparisons = 0usize;
            let mut matches = 0usize;
            let mut overlapped = Vec::new();
            for x in items_i.iter().filter(|item| !noisy.contains(*item)) {
                for y in items_j.iter().filter(|item| !noisy.contains(*item)) {
                    let similarity = string_distance_similarity_with(
                        &x.to_lowercase(),
                        &y.to_lowercase(),
                        DistanceMeasure::JaroWinkler,
                    )
                    .score
                    .unwrap_or(0.0);
                    comparisons += 1;
                    if similarity >= distance_threshold {
                        matches += 1;
                        overlapped.push(format!("{x} => {y}"));
                    }
                }
            }

            let score = if comparisons > 0 {
                matches as f64 / comparisons as f64
            } else {
                0.0
            };
            if score >= threshold {
                new_edges.push(
                    GraphEdge::new(
                        graph.nodes[i].id.clone(),
                        graph.nodes[j].id.clone(),
                        "entities_overlap",
                    )
                    .with_property("entities_overlap_score", GraphProperty::Number(score))
                    .with_property("overlapped_items", GraphProperty::TextList(overlapped)),
                );
            }
        }
    }
    for edge in new_edges {
        graph = graph.add_edge(edge);
    }
    graph
}

/// The "noisy" entity strings to exclude from overlap scoring: the top ~5% most frequent items
/// across all nodes (at least one), ties broken by first-seen order — mirroring Python's
/// `_get_noisy_items` over `Counter.most_common`.
fn noisy_entities(entitied: &[(usize, Vec<String>)]) -> BTreeSet<String> {
    let mut order: Vec<String> = Vec::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_, items) in entitied {
        for item in items {
            if !counts.contains_key(item) {
                order.push(item.clone());
            }
            *counts.entry(item.clone()).or_insert(0) += 1;
        }
    }
    let num_unique = order.len();
    if num_unique == 0 {
        return BTreeSet::new();
    }
    // Python: max(1, int(num_unique * 0.05)) — truncate toward zero, then floor of 1.
    let num_noisy = ((num_unique as f64 * 0.05) as usize).max(1);
    let mut ranked: Vec<(usize, &String)> = order.iter().enumerate().collect();
    // Most frequent first; ties keep first-seen order (Counter.most_common semantics).
    ranked.sort_by(|(idx_a, a), (idx_b, b)| counts[*b].cmp(&counts[*a]).then(idx_a.cmp(idx_b)));
    ranked
        .into_iter()
        .take(num_noisy)
        .map(|(_, item)| item.clone())
        .collect()
}

/// The five default scoring rubric descriptions (score 1 → 5), mirroring Python ragas's
/// `DEFAULT_RUBRICS` in `transforms/filters.py`.
const DEFAULT_NODE_FILTER_RUBRICS: [&str; 5] = [
    "The page content is irrelevant or does not align with the main themes or topics of the document summary.",
    "The page content partially aligns with the document summary, but it includes unrelated details or lacks critical information related to the document's main themes.",
    "The page content generally reflects the document summary but may miss key details or lack depth in addressing the main themes.",
    "The page content aligns well with the document summary, covering the main themes and topics with minor gaps or minimal unrelated information.",
    "The page content is highly relevant, accurate, and directly reflects the main themes of the document summary, covering all important details and adding depth to the understanding of the document's topics.",
];

/// LLM-based node filter that drops low-quality chunk nodes, a faithful port of Python ragas's
/// `CustomNodeFilter` (the `node_filter` step in the default testset pipeline).
///
/// Each chunk node is scored 1–5 by the LLM against its **parent document's** `summary` and a
/// rubric (the parent is the source of a `contains` edge into the chunk — this module's
/// doc→chunk relationship; Python uses `child`). Chunks scoring `<= min_score` (default 2) are
/// removed along with their incident edges. A chunk whose parent has no (or empty) `summary` is
/// kept and never scored, matching Python's "no summary → don't filter".
///
/// Faithful to the chunk branch of Python's `custom_filter`; the non-chunk branch (score a node
/// against its *own* summary) is intentionally omitted — the default pipeline only ever filters
/// chunks (`filter_nodes=filter_chunks`), and scoring a document against its own summary would
/// destructively remove it. Nodes are scored sequentially (deterministic for a mock provider);
/// removal mirrors the engine's `generate_execution_plan` path (remove **every** flagged node),
/// not the unused, remove-first-only `transform` fallback.
pub struct CustomNodeFilter {
    llm: Arc<dyn LlmProvider>,
    min_score: i64,
    rubrics: Vec<String>,
}

impl CustomNodeFilter {
    /// Create a filter over `llm` with the Python defaults (`min_score = 2`, default rubrics).
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            llm,
            min_score: 2,
            rubrics: DEFAULT_NODE_FILTER_RUBRICS
                .iter()
                .map(|line| line.to_string())
                .collect(),
        }
    }

    /// Override the cutoff: chunks scoring `<= min_score` are removed.
    pub fn with_min_score(mut self, min_score: i64) -> Self {
        self.min_score = min_score;
        self
    }

    /// Score every chunk node and return the graph with low-scoring chunks (and their incident
    /// edges) removed.
    pub async fn filter(&self, graph: KnowledgeGraph) -> Result<KnowledgeGraph, RagasError> {
        let mut remove = BTreeSet::new();
        for node in &graph.nodes {
            if node.node_type != "chunk" {
                continue;
            }
            let summary = match self.parent_summary(node, &graph) {
                Some(summary) if !summary.trim().is_empty() => summary,
                _ => continue, // no parent summary -> keep, don't score (Python returns False)
            };
            let content = text_property(node, "text").unwrap_or("");
            if self.score_node(&summary, content).await? <= self.min_score {
                remove.insert(node.id.clone());
            }
        }
        Ok(remove_nodes(graph, &remove))
    }

    /// The `summary` of the chunk's parent document (source of a `contains` edge into it).
    /// Uses the first such edge, mirroring Python's `parent_nodes[0]`.
    fn parent_summary(&self, node: &GraphNode, graph: &KnowledgeGraph) -> Option<String> {
        let parent_id = graph
            .edges
            .iter()
            .find(|edge| edge.target_id == node.id && edge.relationship == "contains")
            .map(|edge| &edge.source_id)?;
        text_property(graph.node(parent_id)?, "summary").map(str::to_string)
    }

    async fn score_node(&self, summary: &str, content: &str) -> Result<i64, RagasError> {
        let rubric = self
            .rubrics
            .iter()
            .enumerate()
            .map(|(index, line)| format!("Score {}: {line}", index + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let prompt = format!(
            "Given a document summary and node content, score the content of the node in the 1 to \
5 range using the rubric. Return ONLY JSON of the form {{\"score\": N}} where N is an integer \
1-5.\n\nRUBRIC:\n{rubric}\n\nDOCUMENT SUMMARY:\n{summary}\n\nNODE CONTENT:\n{content}"
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let value = parse_json_block(&response.content, "custom node filter")?;
        value
            .get("score")
            // Lenient: accept an integer, or a float (e.g. 3.0) rounded to the nearest integer.
            .and_then(|score| {
                score
                    .as_i64()
                    .or_else(|| score.as_f64().map(|value| value.round() as i64))
            })
            .ok_or_else(|| RagasError::Parse {
                message: "custom node filter: missing integer 'score'".to_string(),
            })
    }
}

/// Remove the given node ids and every edge incident to them (the analog of Python's
/// `KnowledgeGraph.remove_node`).
fn remove_nodes(graph: KnowledgeGraph, remove: &BTreeSet<String>) -> KnowledgeGraph {
    KnowledgeGraph {
        nodes: graph
            .nodes
            .into_iter()
            .filter(|node| !remove.contains(&node.id))
            .collect(),
        edges: graph
            .edges
            .into_iter()
            .filter(|edge| !remove.contains(&edge.source_id) && !remove.contains(&edge.target_id))
            .collect(),
    }
}

/// One step of a testset-transform pipeline — the runnable analog of Python ragas's
/// `BaseGraphTransformation` subclasses, plus `Parallel`. Build with the constructors
/// ([`GraphTransform::extract`], [`GraphTransform::cosine`], …) and run a list with
/// [`apply_transforms`].
///
/// `Extract`/`Embed` can be restricted to a node type via [`GraphTransform::for_node_type`]
/// — this selects which nodes receive the extraction (the node-level aspect of Python's
/// `filter_nodes`); all edges are preserved. The other steps apply to the whole graph.
pub enum GraphTransform {
    /// Run an [`LlmExtractor`] over the selected nodes, writing each result as a node property.
    Extract {
        extractor: LlmExtractor,
        node_type: Option<String>,
    },
    /// Run an [`EmbeddingExtractor`] over the selected nodes.
    Embed {
        extractor: EmbeddingExtractor,
        node_type: Option<String>,
    },
    /// Build cosine-similarity relationships ([`build_cosine_relationships`]).
    Cosine { threshold: f64 },
    /// Build entity-overlap relationships ([`build_overlap_relationships`]).
    Overlap {
        distance_threshold: f64,
        threshold: f64,
    },
    /// Drop low-quality chunks ([`CustomNodeFilter`]).
    Filter(CustomNodeFilter),
    /// A group applied as a unit. Mirroring Python's `apply_transforms`, the children run
    /// **sequentially** (Python's `Parallel` only interleaves per-node coroutines elsewhere;
    /// `apply_transforms` itself recurses into the children as a sequence). The result graph is
    /// identical to concurrent execution because grouped transforms are independent. Nesting is
    /// supported (handled by recursion) but expected to be shallow, as in the default pipeline.
    Parallel(Vec<GraphTransform>),
}

impl GraphTransform {
    /// An LLM property extractor over all nodes (restrict with [`Self::for_node_type`]).
    pub fn extract(extractor: LlmExtractor) -> Self {
        Self::Extract {
            extractor,
            node_type: None,
        }
    }

    /// An embedding extractor over all nodes. Restrict with [`Self::for_node_type`] to nodes
    /// that carry text — [`EmbeddingExtractor`] errors on a text-less node (unlike the lenient
    /// [`LlmExtractor`]).
    pub fn embed(extractor: EmbeddingExtractor) -> Self {
        Self::Embed {
            extractor,
            node_type: None,
        }
    }

    /// A cosine-similarity relationship builder.
    pub fn cosine(threshold: f64) -> Self {
        Self::Cosine { threshold }
    }

    /// An entity-overlap relationship builder.
    pub fn overlap(distance_threshold: f64, threshold: f64) -> Self {
        Self::Overlap {
            distance_threshold,
            threshold,
        }
    }

    /// A chunk-quality filter step.
    pub fn filter(filter: CustomNodeFilter) -> Self {
        Self::Filter(filter)
    }

    /// A group of transforms applied as a unit (see [`Self::Parallel`]).
    pub fn parallel(children: Vec<GraphTransform>) -> Self {
        Self::Parallel(children)
    }

    /// Restrict an `Extract`/`Embed` step to nodes of the given type (no-op for others).
    pub fn for_node_type(mut self, node_type: impl Into<String>) -> Self {
        match &mut self {
            Self::Extract { node_type: nt, .. } | Self::Embed { node_type: nt, .. } => {
                *nt = Some(node_type.into());
            }
            _ => {}
        }
        self
    }
}

/// Apply a pipeline of [`GraphTransform`]s to a knowledge graph in order, threading the graph
/// through each step — the runnable analog of Python ragas's `apply_transforms`.
pub async fn apply_transforms(
    mut graph: KnowledgeGraph,
    transforms: Vec<GraphTransform>,
) -> Result<KnowledgeGraph, RagasError> {
    for transform in transforms {
        graph = apply_transform(transform, graph).await?;
    }
    Ok(graph)
}

async fn apply_transform(
    transform: GraphTransform,
    mut graph: KnowledgeGraph,
) -> Result<KnowledgeGraph, RagasError> {
    match transform {
        GraphTransform::Parallel(children) => Box::pin(apply_transforms(graph, children)).await,
        GraphTransform::Cosine { threshold } => build_cosine_relationships(graph, threshold),
        GraphTransform::Overlap {
            distance_threshold,
            threshold,
        } => Ok(build_overlap_relationships(
            graph,
            distance_threshold,
            threshold,
        )),
        GraphTransform::Filter(filter) => filter.filter(graph).await,
        GraphTransform::Extract {
            extractor,
            node_type,
        } => {
            // Collect-then-write within one step: every selected node is extracted against the
            // step's input graph, then results are written back. Between steps the updated graph
            // is threaded onward, so a later step sees an earlier step's writes. (`node_type` is
            // None = all nodes; Some = only that type, mirroring Python's `filter_nodes`.)
            let mut updates = Vec::new();
            for node in &graph.nodes {
                if node_type
                    .as_ref()
                    .is_none_or(|wanted| &node.node_type == wanted)
                {
                    updates.push((node.id.clone(), extractor.extract(node).await?));
                }
            }
            apply_property_updates(&mut graph, updates);
            Ok(graph)
        }
        GraphTransform::Embed {
            extractor,
            node_type,
        } => {
            let mut updates = Vec::new();
            for node in &graph.nodes {
                if node_type
                    .as_ref()
                    .is_none_or(|wanted| &node.node_type == wanted)
                {
                    updates.push((node.id.clone(), extractor.extract(node).await?));
                }
            }
            apply_property_updates(&mut graph, updates);
            Ok(graph)
        }
    }
}

/// Write extracted `(node_id, (property_name, value))` results back onto their nodes. Each id was
/// collected from this same graph in the same step (nothing removes nodes in between), so the
/// `find` always matches; the guard is purely defensive.
fn apply_property_updates(
    graph: &mut KnowledgeGraph,
    updates: Vec<(String, (String, GraphProperty))>,
) {
    for (id, (name, value)) in updates {
        if let Some(node) = graph.nodes.iter_mut().find(|node| node.id == id) {
            node.properties.insert(name, value);
        }
    }
}

/// Generate up to `num_personas` personas from a knowledge graph by clustering similar node
/// summaries — a faithful port of Python ragas's `generate_personas_from_kg`.
///
/// Eligible nodes are those carrying both a `summary` ([`GraphProperty::Text`]) and a
/// `summary_embedding` ([`GraphProperty::Vector`], e.g. from an [`EmbeddingExtractor`] with
/// `property_name = "summary_embedding"`, `embed_property_name = "summary"`). Their embeddings
/// are greedily clustered (cosine `> 0.75`); each cluster's **longest** summary is its
/// representative, and the first `num_personas` representatives are turned into [`Persona`]s by
/// the LLM (`{name, role_description}` → `Persona { name, role: role_description, goals: [] }`).
///
/// Returns the KG-derived personas (vs the manual seed [`PersonaGenerator`]). **Documented
/// divergences:** Python pads with `np.random` duplicates when there are fewer clusters than
/// `num_personas` — we don't (RNG is a non-goal, and padding only repeats personas), so the
/// result may be shorter than `num_personas`; ties for the longest summary resolve to the
/// first (Python's `max(key=len)` semantics). Errors if no node is eligible.
pub async fn generate_personas_from_kg(
    llm: Arc<dyn LlmProvider>,
    graph: &KnowledgeGraph,
    num_personas: usize,
) -> Result<Vec<Persona>, RagasError> {
    let eligible: Vec<(&str, &Vec<f32>)> = graph
        .nodes
        .iter()
        .filter_map(|node| {
            let summary = text_property(node, "summary")?;
            match node.properties.get("summary_embedding") {
                Some(GraphProperty::Vector(embedding)) => Some((summary, embedding)),
                _ => None,
            }
        })
        .collect();
    if eligible.is_empty() {
        return Err(RagasError::Parse {
            message: "persona generation: no node has both a summary and a summary_embedding"
                .to_string(),
        });
    }

    // Greedy clustering: each unvisited node seeds a group with every later node whose summary
    // embedding has cosine similarity > 0.75.
    let count = eligible.len();
    let mut visited = vec![false; count];
    let mut representatives: Vec<&str> = Vec::new();
    for i in 0..count {
        if visited[i] {
            continue;
        }
        visited[i] = true;
        let mut longest = eligible[i].0;
        for j in (i + 1)..count {
            if !visited[j] && cosine_similarity(eligible[i].1, eligible[j].1) > 0.75 {
                visited[j] = true;
                // Longest summary is the representative; keep the first on a length tie.
                if eligible[j].0.len() > longest.len() {
                    longest = eligible[j].0;
                }
            }
        }
        representatives.push(longest);
    }
    representatives.truncate(num_personas);

    let mut personas = Vec::with_capacity(representatives.len());
    for summary in representatives {
        personas.push(generate_persona(&llm, summary).await?);
    }
    Ok(personas)
}

async fn generate_persona(
    llm: &Arc<dyn LlmProvider>,
    summary: &str,
) -> Result<Persona, RagasError> {
    let prompt = format!(
        "Using the provided summary, generate a single persona who would likely interact with or \
benefit from the content. Include a unique name and a concise role description of who they are. \
Return ONLY JSON of the form {{\"name\": \"...\", \"role_description\": \"...\"}}.\n\nSUMMARY:\n{summary}"
    );
    let response = llm
        .generate(LlmRequest {
            messages: vec![ChatMessage::user(prompt)],
            // Python uses temperature 1.0 here for persona diversity.
            temperature: Some(1.0),
        })
        .await?;
    let value = parse_json_block(&response.content, "persona generation")?;
    let field = |key: &str| -> Result<String, RagasError> {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string)
            .ok_or_else(|| RagasError::Parse {
                message: format!("persona generation: missing non-empty '{key}'"),
            })
    };
    Ok(Persona {
        name: field("name")?,
        role: field("role_description")?,
        goals: Vec::new(),
    })
}

/// The length variants a generated query can take (Python ragas `QueryLength`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryLength {
    Long,
    Medium,
    Short,
}

impl QueryLength {
    /// All variants, in the order used for deterministic rotation.
    pub const ALL: [QueryLength; 3] = [QueryLength::Long, QueryLength::Medium, QueryLength::Short];

    /// The string passed to the generation prompt (matches the Python enum values).
    pub fn as_str(self) -> &'static str {
        match self {
            QueryLength::Long => "long",
            QueryLength::Medium => "medium",
            QueryLength::Short => "short",
        }
    }

    /// The enum-variant name recorded on generated samples (Python's `QueryLength.<X>.name`,
    /// e.g. `"LONG"`) — distinct from the lowercase prompt value returned by [`as_str`](Self::as_str).
    pub fn name(self) -> &'static str {
        match self {
            QueryLength::Long => "LONG",
            QueryLength::Medium => "MEDIUM",
            QueryLength::Short => "SHORT",
        }
    }
}

/// The phrasing style a generated query can take (Python ragas `QueryStyle`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStyle {
    Misspelled,
    PerfectGrammar,
    PoorGrammar,
    WebSearchLike,
}

impl QueryStyle {
    /// All variants, in the order used for deterministic rotation.
    pub const ALL: [QueryStyle; 4] = [
        QueryStyle::Misspelled,
        QueryStyle::PerfectGrammar,
        QueryStyle::PoorGrammar,
        QueryStyle::WebSearchLike,
    ];

    /// The string passed to the generation prompt (matches the Python enum values).
    pub fn as_str(self) -> &'static str {
        match self {
            QueryStyle::Misspelled => "Misspelled queries",
            QueryStyle::PerfectGrammar => "Perfect grammar",
            QueryStyle::PoorGrammar => "Poor grammar",
            QueryStyle::WebSearchLike => "Web search like queries",
        }
    }

    /// The enum-variant name recorded on generated samples (Python's `QueryStyle.<X>.name`,
    /// e.g. `"PERFECT_GRAMMAR"`) — distinct from the prompt value returned by [`as_str`](Self::as_str).
    pub fn name(self) -> &'static str {
        match self {
            QueryStyle::Misspelled => "MISSPELLED",
            QueryStyle::PerfectGrammar => "PERFECT_GRAMMAR",
            QueryStyle::PoorGrammar => "POOR_GRAMMAR",
            QueryStyle::WebSearchLike => "WEB_SEARCH_LIKE",
        }
    }
}

/// A single-hop test-generation scenario: one knowledge-graph node, a `term` (theme) drawn from
/// the node, a [`Persona`] that cares about that term, and a query style/length — the runnable
/// analog of Python ragas's `SingleHopScenario`. Produced by [`prepare_single_hop_scenarios`]
/// and consumed by the (forthcoming) sample generator.
#[derive(Debug, Clone, PartialEq)]
pub struct SingleHopScenario {
    pub node_id: String,
    pub term: String,
    pub persona: Persona,
    pub style: QueryStyle,
    pub length: QueryLength,
}

/// Ask the LLM which themes each persona cares about, a faithful port of Python ragas's
/// `ThemesPersonasMatchingPrompt`. Returns a `{persona_name: [relevant_themes]}` map.
pub async fn match_themes_to_personas(
    llm: &Arc<dyn LlmProvider>,
    themes: &[String],
    personas: &[Persona],
) -> Result<BTreeMap<String, Vec<String>>, RagasError> {
    let theme_lines = themes
        .iter()
        .map(|theme| format!("- {theme}"))
        .collect::<Vec<_>>()
        .join("\n");
    let persona_lines = personas
        .iter()
        .map(|persona| format!("- {}: {}", persona.name, persona.role))
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        "Given a list of themes and personas with their roles, associate each persona with the \
relevant themes based on their role description. Return ONLY JSON of the form {{\"mapping\": \
{{\"<persona name>\": [\"<theme>\", ...]}}}}.\n\nTHEMES:\n{theme_lines}\n\nPERSONAS:\n{persona_lines}"
    );
    let response = llm
        .generate(LlmRequest {
            messages: vec![ChatMessage::user(prompt)],
            temperature: Some(0.0),
        })
        .await?;
    let value = parse_json_block(&response.content, "theme-persona matching")?;
    let mapping = value
        .get("mapping")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| RagasError::Parse {
            message: "theme-persona matching: missing 'mapping' object".to_string(),
        })?;
    let mut result = BTreeMap::new();
    for (name, themes) in mapping {
        // Each value must be a list of strings (Python's pydantic `Dict[str, List[str]]` rejects
        // a non-array); erroring beats silently dropping a malformed mapping into an empty list.
        let themes = themes
            .as_array()
            .ok_or_else(|| RagasError::Parse {
                message: format!("theme-persona matching: themes for '{name}' is not an array"),
            })?
            .iter()
            .filter_map(|theme| theme.as_str().map(str::to_string))
            .collect();
        result.insert(name.clone(), themes);
    }
    Ok(result)
}

/// Prepare up to `n` single-hop scenarios from a knowledge graph and a persona list — the
/// deterministic analog of Python ragas's `SingleHopSpecificQuerySynthesizer._generate_scenarios`.
///
/// Nodes are selected by majority type among those carrying an `entities`
/// [`GraphProperty::TextList`] (chunks vs documents; ties favor documents, as in Python). Each
/// selected node contributes up to `ceil(n / nodes)` scenarios: its entities are matched to
/// personas via [`match_themes_to_personas`], and each term that some persona cares about yields
/// one scenario paired with the first such persona. Query style/length rotate deterministically
/// across the produced scenarios.
///
/// **Documented divergence:** Python builds the full `term × persona × style × length` Cartesian
/// product and `random.shuffle`s it before sampling; we don't (RNG is a non-goal and the style/
/// length axes are query-phrasing variety) — we emit one scenario per matched `(node, term)` with
/// rotating style/length, deterministically, capped at `n`. Errors if no node has entities.
pub async fn prepare_single_hop_scenarios(
    llm: &Arc<dyn LlmProvider>,
    graph: &KnowledgeGraph,
    personas: &[Persona],
    n: usize,
) -> Result<Vec<SingleHopScenario>, RagasError> {
    let nodes = select_entity_nodes(graph);
    if nodes.is_empty() {
        return Err(RagasError::Parse {
            message: "single-hop scenarios: no node has an `entities` property".to_string(),
        });
    }
    let samples_per_node = n.div_ceil(nodes.len());

    let mut scenarios = Vec::new();
    for node in nodes {
        if scenarios.len() >= n {
            break;
        }
        let themes = match node.properties.get("entities") {
            Some(GraphProperty::TextList(themes)) if !themes.is_empty() => themes,
            _ => continue,
        };
        let mapping = match_themes_to_personas(llm, themes, personas).await?;

        let mut per_node = 0;
        for term in themes {
            if scenarios.len() >= n || per_node >= samples_per_node {
                break;
            }
            // The first persona whose matched themes include this term (case-insensitive).
            let matched = personas.iter().find(|persona| {
                mapping
                    .get(&persona.name)
                    .is_some_and(|themes| themes.iter().any(|t| t.eq_ignore_ascii_case(term)))
            });
            if let Some(persona) = matched {
                let index = scenarios.len();
                scenarios.push(SingleHopScenario {
                    node_id: node.id.clone(),
                    term: term.clone(),
                    persona: persona.clone(),
                    style: QueryStyle::ALL[index % QueryStyle::ALL.len()],
                    length: QueryLength::ALL[index % QueryLength::ALL.len()],
                });
                per_node += 1;
            }
        }
    }
    Ok(scenarios)
}

/// Select the nodes to draw single-hop scenarios from: those carrying an `entities`
/// [`GraphProperty::TextList`], restricted to the majority node type (chunks vs documents; ties
/// favor documents, matching Python's `get_node_clusters`).
fn select_entity_nodes(graph: &KnowledgeGraph) -> Vec<&GraphNode> {
    let has_entities = |node: &GraphNode| matches!(node.properties.get("entities"), Some(GraphProperty::TextList(items)) if !items.is_empty());
    let mut chunks = 0usize;
    let mut documents = 0usize;
    for node in &graph.nodes {
        if has_entities(node) {
            match node.node_type.as_str() {
                "chunk" => chunks += 1,
                "document" => documents += 1,
                _ => {}
            }
        }
    }
    let wanted = if chunks > documents {
        "chunk"
    } else {
        "document"
    };
    graph
        .nodes
        .iter()
        .filter(|node| node.node_type == wanted && has_entities(node))
        .collect()
}

/// Generate a single-hop query + grounded answer for one scenario — a faithful port of Python
/// ragas's `QueryAnswerGenerationPrompt`. The query is phrased from the persona's perspective,
/// incorporates the scenario's `term`, and follows the requested style/length; the answer is drawn
/// *only* from `context`. An optional `llm_context` adds guidance on the kind of question to ask.
/// Returns `(query, answer)`; errors if the model omits either field.
async fn generate_single_hop_query_answer(
    llm: &Arc<dyn LlmProvider>,
    scenario: &SingleHopScenario,
    context: &str,
    llm_context: Option<&str>,
) -> Result<(String, String), RagasError> {
    let extra = match llm_context {
        Some(guidance) if !guidance.trim().is_empty() => format!(
            "\n3. **Additional Context**: Use the following guidance for the kind of question to \
generate and how to structure the answer, while still drawing all content only from the context: \
{guidance}"
        ),
        _ => String::new(),
    };
    let prompt = format!(
        "Generate a single-hop query and answer based on the specified conditions (persona, term, \
style, length) and the provided context. Ensure the answer is entirely faithful to the context, \
using only the information directly from the provided context.\n\
### Instructions:\n\
1. **Generate a Query**: Based on the context, persona, term, style, and length, create a \
question that aligns with the persona's perspective and incorporates the term.\n\
2. **Generate an Answer**: Using only the content from the provided context, construct a detailed \
answer to the query. Do not add any information not included in or inferable from the context.{extra}\n\n\
PERSONA: {persona_name} — {persona_role}\n\
TERM: {term}\n\
STYLE: {style}\n\
LENGTH: {length}\n\
CONTEXT:\n{context}\n\n\
Return ONLY JSON of the form {{\"query\": \"...\", \"answer\": \"...\"}}.",
        persona_name = scenario.persona.name,
        persona_role = scenario.persona.role,
        term = scenario.term,
        style = scenario.style.as_str(),
        length = scenario.length.as_str(),
    );
    let response = llm
        .generate(LlmRequest {
            messages: vec![ChatMessage::user(prompt)],
            temperature: Some(0.0),
        })
        .await?;
    let value = parse_json_block(&response.content, "single-hop query/answer generation")?;
    let field = |key: &str| -> Result<String, RagasError> {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| RagasError::Parse {
                message: format!("single-hop query/answer generation: missing '{key}' string"),
            })
    };
    Ok((field("query")?, field("answer")?))
}

/// Persona-conditioned single-hop test-set synthesizer — the runnable analog of Python ragas's
/// `SingleHopSpecificQuerySynthesizer`. It prepares scenarios with [`prepare_single_hop_scenarios`]
/// (entity-bearing nodes → theme/persona matching → deterministic style/length rotation) and turns
/// each into a grounded [`SingleTurnSample`] via [`generate_single_hop_query_answer`], producing an
/// [`EvaluationDataset`] end-to-end.
///
/// **Documented divergence:** Python's `_generate_sample` leaves `response`/`retrieved_contexts`
/// empty (those are the system-under-test's job) and only fills `reference` + `reference_contexts`.
/// This crate's [`EvaluationDataset`] requires a non-empty `response` and `retrieved_contexts`, so
/// — matching this module's existing [`Synthesizer`] — we mirror the generated answer into
/// `response` and the node text into `retrieved_contexts`, while *also* recording the faithful
/// `reference`/`reference_contexts`. Scenarios whose node has no usable text are skipped (as the
/// existing `Synthesizer` skips empty chunks) rather than emitting an unevaluatable sample.
pub struct SingleHopSpecificSynthesizer {
    llm: Arc<dyn LlmProvider>,
    llm_context: Option<String>,
}

impl SingleHopSpecificSynthesizer {
    /// Create a synthesizer over the given LLM provider.
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            llm,
            llm_context: None,
        }
    }

    /// Optional guidance string passed to every query/answer generation (Python's `llm_context`),
    /// e.g. "ask comparison questions". Defaults to none.
    pub fn with_llm_context(mut self, llm_context: impl Into<String>) -> Self {
        self.llm_context = Some(llm_context.into());
        self
    }

    /// Generate up to `n` grounded single-hop samples from the knowledge graph and personas.
    ///
    /// Prepares scenarios with [`prepare_single_hop_scenarios`], then for each scenario looks up
    /// its node's text (the reference context) and asks the LLM for a `(query, answer)` pair
    /// grounded in it. Each sample records `synthesis_type`/`source_node_ids`/`term`/`persona_name`/
    /// `query_style`/`query_length` in its metadata. Returns `RagasError::EmptyDataset` when no
    /// usable sample is produced (no matched scenario, or every scenario node lacked text), and
    /// propagates `Err` from scenario prep or generation. Errors if the graph has no entity node.
    pub async fn generate(
        &self,
        graph: &KnowledgeGraph,
        personas: &[Persona],
        n: usize,
    ) -> Result<EvaluationDataset, RagasError> {
        let scenarios = prepare_single_hop_scenarios(&self.llm, graph, personas, n).await?;
        let mut samples = Vec::new();
        for scenario in &scenarios {
            // The reference context is the scenario node's text (Python: nodes[0].page_content).
            // Skip nodes without usable text rather than emit an unevaluatable empty-context sample.
            let context = match graph
                .node(&scenario.node_id)
                .and_then(|node| text_property(node, "text"))
            {
                Some(text) if !text.trim().is_empty() => text.to_string(),
                _ => continue,
            };
            let (query, answer) = generate_single_hop_query_answer(
                &self.llm,
                scenario,
                &context,
                self.llm_context.as_deref(),
            )
            .await?;
            samples.push(
                SingleTurnSample::new(query, answer.clone(), vec![context.clone()])
                    .with_reference(answer)
                    .with_reference_contexts(vec![context])
                    .with_metadata("synthesis_type", "single-hop")
                    .with_metadata("source_node_ids", scenario.node_id.clone())
                    .with_metadata("term", scenario.term.clone())
                    .with_metadata("persona_name", scenario.persona.name.clone())
                    .with_metadata("query_style", scenario.style.name())
                    .with_metadata("query_length", scenario.length.name()),
            );
        }
        EvaluationDataset::new(samples)
    }
}

/// A multi-hop test-generation scenario: a small cluster of knowledge-graph nodes joined by a
/// relationship (entity overlap for the specific synthesizer), a `combination` of themes drawn from
/// that relationship, a [`Persona`] that cares about them, and a query style/length — the runnable
/// analog of Python ragas's `MultiHopScenario`. Produced by [`prepare_multi_hop_specific_scenarios`]
/// and consumed by [`MultiHopSpecificSynthesizer`].
#[derive(Debug, Clone, PartialEq)]
pub struct MultiHopScenario {
    /// The cluster's node ids, in hop order (the generated contexts are tagged `<1-hop>`, `<2-hop>`).
    pub node_ids: Vec<String>,
    /// The themes the query must incorporate (one entity for the specific synthesizer).
    pub combination: Vec<String>,
    pub persona: Persona,
    pub style: QueryStyle,
    pub length: QueryLength,
}

/// Prepare up to `n` multi-hop scenarios from a knowledge graph and persona list — the
/// deterministic analog of Python ragas's `MultiHopSpecificQuerySynthesizer._generate_scenarios`.
///
/// Clusters are the pairs of nodes joined by an `entities_overlap` edge (built by
/// [`build_overlap_relationships`]), normalized so the smaller node id comes first and deduped
/// (Python's `find_two_nodes_single_rel`). Each cluster contributes up to `ceil(n / clusters)`
/// scenarios: the edge's `overlapped_items` (`"x => y"` strings) are split into unique themes,
/// matched to personas via [`match_themes_to_personas`], and each theme that some persona cares
/// about and that at least one cluster node carries as an `entities` item yields one scenario
/// (`combination = [theme]`), paired with the first such persona and the cluster nodes that
/// actually contain the theme. Query style/length rotate deterministically. Errors if no
/// `entities_overlap` edge exists.
///
/// **Documented divergence:** Python builds the full `combination × persona × style × length`
/// product, `random.shuffle`s it, and samples with a diversity heuristic; we don't (RNG is a
/// non-goal) — we emit one scenario per matched theme with rotating style/length, deterministically,
/// capped at `n`. Theme extraction preserves first-seen order (Python uses an unordered `set`).
pub async fn prepare_multi_hop_specific_scenarios(
    llm: &Arc<dyn LlmProvider>,
    graph: &KnowledgeGraph,
    personas: &[Persona],
    n: usize,
) -> Result<Vec<MultiHopScenario>, RagasError> {
    let clusters = entity_overlap_clusters(graph);
    if clusters.is_empty() {
        return Err(RagasError::Parse {
            message: "multi-hop scenarios: no `entities_overlap` edge in the graph".to_string(),
        });
    }
    let samples_per_cluster = n.div_ceil(clusters.len());

    let mut scenarios = Vec::new();
    for cluster in &clusters {
        if scenarios.len() >= n {
            break;
        }
        let themes = extract_overlap_themes(&cluster.overlapped_items);
        if themes.is_empty() {
            continue;
        }
        let mapping = match_themes_to_personas(llm, &themes, personas).await?;
        let cluster_nodes = [cluster.node_a.as_str(), cluster.node_b.as_str()];

        let mut per_cluster = 0;
        for theme in &themes {
            if scenarios.len() >= n || per_cluster >= samples_per_cluster {
                break;
            }
            // First persona whose matched themes include this theme (case-insensitive).
            let persona = personas.iter().find(|persona| {
                mapping
                    .get(&persona.name)
                    .is_some_and(|themes| themes.iter().any(|t| t.eq_ignore_ascii_case(theme)))
            });
            let Some(persona) = persona else { continue };
            // Cluster nodes that actually carry this theme as an `entities` item (Python's
            // `valid_nodes`); skip the theme if neither node does.
            let node_ids: Vec<String> = cluster_nodes
                .iter()
                .filter(|id| node_has_entity(graph, id, theme))
                .map(|id| id.to_string())
                .collect();
            if node_ids.is_empty() {
                continue;
            }
            let index = scenarios.len();
            scenarios.push(MultiHopScenario {
                node_ids,
                combination: vec![theme.clone()],
                persona: persona.clone(),
                style: QueryStyle::ALL[index % QueryStyle::ALL.len()],
                length: QueryLength::ALL[index % QueryLength::ALL.len()],
            });
            per_cluster += 1;
        }
    }
    Ok(scenarios)
}

/// A normalized entity-overlap cluster: two node ids (smaller first) and the edge's overlap items.
struct OverlapCluster {
    node_a: String,
    node_b: String,
    overlapped_items: Vec<String>,
}

/// Find the `entities_overlap` clusters: each edge between two distinct nodes, normalized so the
/// smaller node id is first and deduped by the unordered node pair (Python `find_two_nodes_single_rel`).
fn entity_overlap_clusters(graph: &KnowledgeGraph) -> Vec<OverlapCluster> {
    let mut seen = BTreeSet::new();
    let mut clusters = Vec::new();
    for edge in graph.edges_by_relationship("entities_overlap") {
        if edge.source_id == edge.target_id {
            continue;
        }
        let (node_a, node_b) = if edge.source_id <= edge.target_id {
            (edge.source_id.clone(), edge.target_id.clone())
        } else {
            (edge.target_id.clone(), edge.source_id.clone())
        };
        if !seen.insert((node_a.clone(), node_b.clone())) {
            continue;
        }
        let overlapped_items = match edge.properties.get("overlapped_items") {
            Some(GraphProperty::TextList(items)) => items.clone(),
            _ => Vec::new(),
        };
        clusters.push(OverlapCluster {
            node_a,
            node_b,
            overlapped_items,
        });
    }
    clusters
}

/// Extract the unique theme names from `overlapped_items` (`"x => y"` strings → both `x` and `y`),
/// preserving first-seen order — the analog of Python's `_extract_themes_from_overlaps`.
fn extract_overlap_themes(overlapped_items: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut themes = Vec::new();
    for item in overlapped_items {
        for side in item.split(" => ") {
            let side = side.trim();
            if !side.is_empty() && seen.insert(side.to_string()) {
                themes.push(side.to_string());
            }
        }
    }
    themes
}

/// Whether `node_id`'s `entities` list contains `theme` (case-insensitive).
fn node_has_entity(graph: &KnowledgeGraph, node_id: &str, theme: &str) -> bool {
    matches!(
        graph.node(node_id).and_then(|node| node.properties.get("entities")),
        Some(GraphProperty::TextList(items))
            if items.iter().any(|item| item.eq_ignore_ascii_case(theme))
    )
}

/// Generate a multi-hop query + grounded answer for one scenario — a faithful port of Python ragas's
/// multi-hop `QueryAnswerGenerationPrompt`. The query must combine information across the hop-tagged
/// `contexts` and explicitly incorporate the scenario's themes; the answer is drawn *only* from the
/// contexts. An optional `llm_context` adds guidance. Returns `(query, answer)`.
async fn generate_multi_hop_query_answer(
    llm: &Arc<dyn LlmProvider>,
    scenario: &MultiHopScenario,
    contexts: &[String],
    llm_context: Option<&str>,
) -> Result<(String, String), RagasError> {
    let extra = match llm_context {
        Some(guidance) if !guidance.trim().is_empty() => format!(
            "\n4. **Additional Context**: Use the following guidance for the kind of question to \
generate and how to structure the answer, while still drawing all content only from the contexts: \
{guidance}"
        ),
        _ => String::new(),
    };
    let prompt = format!(
        "Generate a multi-hop query and answer based on the specified conditions (persona, themes, \
style, length) and the provided context. The themes are phrases extracted from the context that \
highlight its suitability for multi-hop query creation; ensure the query explicitly incorporates \
them.\n\
### Instructions:\n\
1. **Generate a Multi-Hop Query**: Use the provided context segments and themes to form a query \
that requires combining information from multiple segments (e.g. <1-hop> and <2-hop>). Ensure the \
query explicitly incorporates one or more themes and reflects their relevance to the context.\n\
2. **Generate an Answer**: Use only the content from the provided context to create a detailed and \
faithful answer. Do not add any information not directly present or inferable from the context.\n\
3. **Multi-Hop Context Tags**: each context segment is tagged <1-hop>, <2-hop>, etc.; the query \
must use information from at least two segments and connect them meaningfully.{extra}\n\n\
PERSONA: {persona_name} — {persona_role}\n\
THEMES: {themes}\n\
STYLE: {style}\n\
LENGTH: {length}\n\
CONTEXT:\n{context}\n\n\
Return ONLY JSON of the form {{\"query\": \"...\", \"answer\": \"...\"}}.",
        persona_name = scenario.persona.name,
        persona_role = scenario.persona.role,
        themes = scenario.combination.join(", "),
        style = scenario.style.as_str(),
        length = scenario.length.as_str(),
        context = contexts.join("\n\n"),
    );
    let response = llm
        .generate(LlmRequest {
            messages: vec![ChatMessage::user(prompt)],
            temperature: Some(0.0),
        })
        .await?;
    let value = parse_json_block(&response.content, "multi-hop query/answer generation")?;
    let field = |key: &str| -> Result<String, RagasError> {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| RagasError::Parse {
                message: format!("multi-hop query/answer generation: missing '{key}' string"),
            })
    };
    Ok((field("query")?, field("answer")?))
}

/// Build the hop-tagged reference contexts for a multi-hop scenario: each node's text prefixed with
/// `<{i+1}-hop>` — a faithful port of Python's `make_contexts`. The hop number is the node's
/// position in the cluster (`enumerate`, never renumbered), and a node with missing/empty text still
/// yields its tag-only `"<N-hop>\n\n"` entry (matching Python's `.get("page_content", "")`); the tag
/// prefix keeps every context non-blank, so the dataset's non-empty-context invariant still holds.
fn multi_hop_contexts(graph: &KnowledgeGraph, scenario: &MultiHopScenario) -> Vec<String> {
    scenario
        .node_ids
        .iter()
        .enumerate()
        .map(|(i, node_id)| {
            let text = graph
                .node(node_id)
                .and_then(|node| text_property(node, "text"))
                .unwrap_or("");
            format!("<{hop}-hop>\n\n{text}", hop = i + 1)
        })
        .collect()
}

/// Entity-overlap multi-hop test-set synthesizer — the runnable analog of Python ragas's
/// `MultiHopSpecificQuerySynthesizer`. It prepares scenarios with
/// [`prepare_multi_hop_specific_scenarios`] (entity-overlap clusters → theme/persona matching →
/// deterministic style/length rotation) and turns each into a grounded [`SingleTurnSample`] whose
/// query combines the cluster's hop-tagged contexts, producing an [`EvaluationDataset`].
///
/// **Documented divergence (same as [`SingleHopSpecificSynthesizer`]):** Python's `_generate_sample`
/// sets only `reference` + `reference_contexts`; this crate's [`EvaluationDataset`] requires a
/// non-empty `response`/`retrieved_contexts`, so the generated answer and contexts are mirrored into
/// them. Contexts follow Python's `make_contexts` exactly (one tag per node, hop = node position), so
/// a textless node keeps its tag-only slot rather than being dropped.
pub struct MultiHopSpecificSynthesizer {
    llm: Arc<dyn LlmProvider>,
    llm_context: Option<String>,
}

impl MultiHopSpecificSynthesizer {
    /// Create a synthesizer over the given LLM provider.
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            llm,
            llm_context: None,
        }
    }

    /// Optional guidance string passed to every query/answer generation (Python's `llm_context`),
    /// e.g. "ask cause-effect questions". Defaults to none.
    pub fn with_llm_context(mut self, llm_context: impl Into<String>) -> Self {
        self.llm_context = Some(llm_context.into());
        self
    }

    /// Generate up to `n` grounded multi-hop samples from the knowledge graph and personas.
    ///
    /// Prepares scenarios with [`prepare_multi_hop_specific_scenarios`], then for each scenario
    /// builds the hop-tagged contexts and asks the LLM for a `(query, answer)` pair that combines
    /// them. Each sample records `synthesis_type`/`source_node_ids`/`themes`/`persona_name`/
    /// `query_style`/`query_length` metadata. Returns `RagasError::EmptyDataset` when no usable
    /// sample is produced, and propagates `Err` from scenario prep or generation. Errors if the
    /// graph has no `entities_overlap` edge.
    pub async fn generate(
        &self,
        graph: &KnowledgeGraph,
        personas: &[Persona],
        n: usize,
    ) -> Result<EvaluationDataset, RagasError> {
        let scenarios = prepare_multi_hop_specific_scenarios(&self.llm, graph, personas, n).await?;
        let mut samples = Vec::new();
        for scenario in &scenarios {
            // Hop-tagged contexts (one per cluster node). Empty only if a scenario somehow has no
            // nodes, which scenario prep prevents; guarded defensively so it can't poison the dataset.
            let contexts = multi_hop_contexts(graph, scenario);
            if contexts.is_empty() {
                continue;
            }
            let (query, answer) = generate_multi_hop_query_answer(
                &self.llm,
                scenario,
                &contexts,
                self.llm_context.as_deref(),
            )
            .await?;
            samples.push(
                SingleTurnSample::new(query, answer.clone(), contexts.clone())
                    .with_reference(answer)
                    .with_reference_contexts(contexts)
                    .with_metadata("synthesis_type", "multi-hop")
                    .with_metadata("source_node_ids", scenario.node_ids.join(","))
                    .with_metadata("themes", scenario.combination.join(", "))
                    .with_metadata("persona_name", scenario.persona.name.clone())
                    .with_metadata("query_style", scenario.style.name())
                    .with_metadata("query_length", scenario.length.name()),
            );
        }
        EvaluationDataset::new(samples)
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

    #[test]
    fn test_13_2_1_splitters_produce_stable_chunks_with_source_metadata() {
        // SCEN-13.2.1 / AC1 / TEST-13.2.1
        let chunks = split_text_into_chunks(
            "doc-1",
            "Ragas evaluates retrieval. It scores answers with context.",
            26,
        );

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].id, "doc-1-chunk-0");
        assert_eq!(chunks[0].source_id, "doc-1");
        assert_eq!(
            chunks[0].metadata.get("source_id").map(String::as_str),
            Some("doc-1")
        );
        assert_eq!(
            chunks[1].metadata.get("chunk_index").map(String::as_str),
            Some("1")
        );
        assert!(chunks.iter().all(|chunk| chunk.text.len() <= 26));
    }

    #[test]
    fn test_13_2_2_extractors_attach_entities_themes_and_summaries() {
        // SCEN-13.2.2 / AC2 / TEST-13.2.2
        let graph =
            KnowledgeGraph::new().add_node(GraphNode::new("doc-1-chunk-0", "chunk").with_property(
                "text",
                GraphProperty::Text("RAG evaluates retrieval".to_string()),
            ));
        let updated = attach_extractions(
            graph,
            "doc-1-chunk-0",
            ExtractionBundle::new(
                vec!["RAG".to_string(), "retrieval".to_string()],
                vec!["evaluation".to_string()],
                "Chunk about retrieval evaluation",
            ),
        );
        let node = updated.node("doc-1-chunk-0").expect("chunk node");

        assert_eq!(
            node.properties.get("entities"),
            Some(&GraphProperty::TextList(vec![
                "RAG".to_string(),
                "retrieval".to_string()
            ]))
        );
        assert_eq!(
            node.properties.get("themes"),
            Some(&GraphProperty::TextList(vec!["evaluation".to_string()]))
        );
        assert_eq!(
            node.properties.get("summary"),
            Some(&GraphProperty::Text(
                "Chunk about retrieval evaluation".to_string()
            ))
        );
    }

    #[test]
    fn test_13_2_3_relationship_builders_create_deterministic_edges() {
        // SCEN-13.2.3 / AC3 / TEST-13.2.3
        let chunks = split_text_into_chunks(
            "doc-1",
            "Ragas evaluates retrieval. It scores answers with context.",
            26,
        );
        assert_eq!(chunks.len(), 3);
        let graph = chunks.iter().fold(
            KnowledgeGraph::new().add_node(GraphNode::new("doc-1", "document")),
            |graph, chunk| graph.add_node(chunk.to_graph_node()),
        );
        let graph = build_chunk_relationships(graph, "doc-1", &chunks);

        let contains = graph.edges_by_relationship("contains");
        assert_eq!(contains.len(), chunks.len());
        assert_eq!(contains[0].source_id, "doc-1");
        assert_eq!(contains[0].target_id, "doc-1-chunk-0");
        assert_eq!(
            contains[0].properties.get("order"),
            Some(&GraphProperty::Number(0.0))
        );

        let next = graph.edges_by_relationship("next");
        assert_eq!(next.len(), chunks.len() - 1);
        assert_eq!(next[0].source_id, "doc-1-chunk-0");
        assert_eq!(next[0].target_id, "doc-1-chunk-1");
    }

    fn synthesizer_graph() -> KnowledgeGraph {
        KnowledgeGraph::new()
            .add_node(
                GraphNode::new("chunk-1", "chunk")
                    .with_property(
                        "text",
                        GraphProperty::Text("RAG evaluates retrieval".to_string()),
                    )
                    .with_property(
                        "summary",
                        GraphProperty::Text("retrieval evaluation summary".to_string()),
                    ),
            )
            .add_node(
                GraphNode::new("chunk-2", "chunk")
                    .with_property(
                        "text",
                        GraphProperty::Text("LLM judges score answers".to_string()),
                    )
                    .with_property(
                        "summary",
                        GraphProperty::Text("answer scoring summary".to_string()),
                    ),
            )
            .add_edge(
                GraphEdge::new("chunk-1", "chunk-2", "next")
                    .with_property("order", GraphProperty::Number(0.0)),
            )
    }

    #[test]
    fn test_13_3_1_persona_generator_stores_name_role_and_goals() {
        // SCEN-13.3.1 / AC1 / TEST-13.3.1
        let persona = PersonaGenerator::new("deterministic-seed").generate(
            "QA Lead",
            "evaluation engineer",
            vec![
                "verify grounding".to_string(),
                "stress retrieval".to_string(),
            ],
        );

        assert_eq!(persona.name, "QA Lead");
        assert_eq!(persona.role, "evaluation engineer");
        assert_eq!(
            persona.goals,
            vec![
                "verify grounding".to_string(),
                "stress retrieval".to_string()
            ]
        );
    }

    #[test]
    fn test_13_3_2_single_hop_synthesizer_creates_samples_from_one_chunk() {
        // SCEN-13.3.2 / AC2 / TEST-13.3.2
        let graph = synthesizer_graph();
        let persona = PersonaGenerator::new("deterministic-seed").generate(
            "Research Analyst",
            "domain evaluator",
            vec!["ask grounded questions".to_string()],
        );

        let synthesized =
            synthesize_single_hop_sample(&graph, "chunk-1", &persona).expect("single-hop sample");

        assert_eq!(synthesized.hop_count, 1);
        assert_eq!(synthesized.source_node_ids, vec!["chunk-1"]);
        assert_eq!(
            synthesized.sample.retrieved_contexts,
            vec!["RAG evaluates retrieval".to_string()]
        );
        assert!(synthesized.sample.user_input.contains("domain evaluator"));
        assert!(
            synthesized
                .sample
                .response
                .contains("retrieval evaluation summary")
        );
        assert_eq!(
            synthesized
                .sample
                .metadata
                .get("synthesis_type")
                .map(String::as_str),
            Some("single-hop")
        );
    }

    #[test]
    fn test_13_3_3_multi_hop_synthesizer_combines_related_graph_nodes() {
        // SCEN-13.3.3 / AC3 / TEST-13.3.3
        let graph = synthesizer_graph();
        let persona = PersonaGenerator::new("deterministic-seed").generate(
            "Research Analyst",
            "domain evaluator",
            vec!["compare related evidence".to_string()],
        );

        let synthesized = synthesize_multi_hop_sample(&graph, "chunk-1", "next", &persona)
            .expect("multi-hop sample");

        assert_eq!(synthesized.hop_count, 2);
        assert_eq!(synthesized.source_node_ids, vec!["chunk-1", "chunk-2"]);
        assert_eq!(synthesized.sample.retrieved_contexts.len(), 2);
        assert!(synthesized.sample.user_input.contains("multi-hop"));
        assert!(
            synthesized
                .sample
                .response
                .contains("retrieval evaluation summary")
        );
        assert!(
            synthesized
                .sample
                .response
                .contains("answer scoring summary")
        );
        assert_eq!(
            synthesized
                .sample
                .metadata
                .get("relationship")
                .map(String::as_str),
            Some("next")
        );
    }

    #[test]
    fn test_20_1_1_graph_parity_fixture_roundtrips_nodes_edges_and_properties() {
        // SCEN-20.1.1 / AC1 / TEST-20.1.1
        let input = serde_json::json!({
            "feature": "testset_graph",
            "upstream_commit": "298b682",
            "graph": {
                "nodes": [
                    {
                        "id": "doc-1",
                        "node_type": "document",
                        "properties": {
                            "title": {"Text": "RAG Guide"},
                            "trusted": {"Boolean": true},
                            "page_count": {"Number": 3.0}
                        }
                    }
                ],
                "edges": [
                    {
                        "source_id": "doc-1",
                        "target_id": "chunk-1",
                        "relationship": "contains",
                        "properties": {"order": {"Number": 0.0}}
                    }
                ]
            }
        })
        .to_string();

        let fixture = parse_graph_parity_fixture(&input).expect("graph fixture parses");
        assert_eq!(fixture.feature, "testset_graph");
        assert_eq!(
            fixture
                .graph
                .node("doc-1")
                .and_then(|node| node.properties.get("trusted")),
            Some(&GraphProperty::Boolean(true))
        );

        let encoded = serialize_graph_parity_fixture(&fixture).expect("serialize fixture");
        let roundtrip = parse_graph_parity_fixture(&encoded).expect("roundtrip parses");
        assert_eq!(roundtrip, fixture);
    }

    #[test]
    fn test_20_2_2_extractor_outputs_normalize_into_stable_graph_properties() {
        // SCEN-20.2.2 / AC2 / TEST-20.2.2
        let extractions = ExtractionBundle::new(
            vec![
                "retrieval".to_string(),
                "RAG".to_string(),
                "RAG".to_string(),
            ],
            vec!["evaluation".to_string(), "retrieval".to_string()],
            "Chunk about retrieval evaluation",
        );
        let properties = normalize_extraction_properties(extractions.clone());

        assert_eq!(
            properties.get("entities"),
            Some(&GraphProperty::TextList(vec![
                "RAG".to_string(),
                "retrieval".to_string()
            ]))
        );
        assert_eq!(
            properties.get("themes"),
            Some(&GraphProperty::TextList(vec![
                "evaluation".to_string(),
                "retrieval".to_string()
            ]))
        );
        assert_eq!(
            properties.get("summary"),
            Some(&GraphProperty::Text(
                "Chunk about retrieval evaluation".to_string()
            ))
        );

        let chunks = split_text_into_chunks(
            "doc-1",
            "RAG evaluates retrieval. It scores answers with context.",
            26,
        );
        let graph = chunks.iter().fold(
            KnowledgeGraph::new().add_node(GraphNode::new("doc-1", "document")),
            |graph, chunk| graph.add_node(chunk.to_graph_node()),
        );
        let graph = attach_extractions(graph, "doc-1-chunk-0", extractions);
        let graph = build_chunk_relationships(graph, "doc-1", &chunks);
        let node = graph.node("doc-1-chunk-0").expect("chunk node");

        assert_eq!(node.properties.get("entities"), properties.get("entities"));
        assert_eq!(node.properties.get("themes"), properties.get("themes"));
        assert_eq!(node.properties.get("summary"), properties.get("summary"));

        let contains = graph.edges_by_relationship("contains");
        assert_eq!(contains.len(), chunks.len());
        assert_eq!(
            contains[0].properties.get("order"),
            Some(&GraphProperty::Number(0.0))
        );

        let next = graph.edges_by_relationship("next");
        assert_eq!(next.len(), chunks.len() - 1);
        assert_eq!(next[0].source_id, "doc-1-chunk-0");
        assert_eq!(next[0].target_id, "doc-1-chunk-1");
    }

    #[test]
    fn test_20_3_2_prompt_snapshots_preserve_variables_and_message_order() {
        // SCEN-20.3.2 / AC2 / TEST-20.3.2
        let snapshot = SynthesizerPromptSnapshot {
            strategy: SynthesizerStrategy::SingleHop,
            variables: BTreeMap::from([
                ("chunk_id".to_string(), "chunk-1".to_string()),
                ("persona_role".to_string(), "domain evaluator".to_string()),
                (
                    "summary".to_string(),
                    "retrieval evaluation summary".to_string(),
                ),
            ]),
            messages: vec![
                SynthesizerPromptMessage::new(
                    "system",
                    "Generate a {{strategy}} question without unsupported assumptions.",
                ),
                SynthesizerPromptMessage::new(
                    "user",
                    "Persona: {{persona_role}}\nChunk: {{chunk_id}}\nSummary: {{summary}}",
                ),
            ],
        };

        let rendered = render_synthesizer_prompt_snapshot(&snapshot)
            .expect("snapshot renders deterministically");

        assert_eq!(rendered.strategy, SynthesizerStrategy::SingleHop);
        assert_eq!(rendered.variables, snapshot.variables);
        assert_eq!(rendered.rendered_messages.len(), 2);
        assert_eq!(rendered.rendered_messages[0].role, "system");
        assert_eq!(rendered.rendered_messages[1].role, "user");
        assert_eq!(
            rendered.rendered_messages[0].content,
            "Generate a single-hop question without unsupported assumptions."
        );
        assert_eq!(
            rendered.rendered_messages[1].content,
            "Persona: domain evaluator\nChunk: chunk-1\nSummary: retrieval evaluation summary"
        );

        let graph = synthesizer_graph();
        let persona = PersonaGenerator::new("deterministic-seed").generate(
            "Research Analyst",
            "domain evaluator",
            vec!["ask grounded questions".to_string()],
        );
        let expected =
            synthesize_single_hop_sample(&graph, "chunk-1", &persona).expect("expected sample");
        let actual =
            synthesize_single_hop_sample(&graph, "chunk-1", &persona).expect("actual sample");
        let comparison = compare_synthesized_sample_fixture(&expected, &actual);

        assert!(
            comparison.matches,
            "sample fixture drift: {:?}",
            comparison.drift
        );
        assert_eq!(comparison.drift, None);
    }

    #[test]
    fn test_29_1_1_graph_cluster_and_advanced_query_return_expected_nodes() {
        // SCEN-29.1.1 / AC1 / TEST-29.1.1
        let graph = KnowledgeGraph::new()
            .add_node(
                GraphNode::new("chunk-1", "chunk")
                    .with_property("theme", GraphProperty::Text("retrieval".to_string())),
            )
            .add_node(
                GraphNode::new("chunk-2", "chunk")
                    .with_property("theme", GraphProperty::Text("generation".to_string())),
            )
            .add_node(
                GraphNode::new("chunk-3", "chunk")
                    .with_property("theme", GraphProperty::Text("retrieval".to_string())),
            )
            .add_edge(GraphEdge::new("chunk-1", "chunk-3", "related"));

        let clusters = cluster_graph_by_property(&graph, "theme");
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].key, "generation");
        assert_eq!(clusters[1].key, "retrieval");
        assert_eq!(clusters[1].node_ids, vec!["chunk-1", "chunk-3"]);

        let result = query_graph_advanced(
            &graph,
            &GraphAdvancedQuery::new()
                .with_node_type("chunk")
                .with_property("theme", GraphProperty::Text("retrieval".to_string()))
                .with_outgoing_relationship("related"),
        );
        let ids = result.into_iter().map(|node| node.id).collect::<Vec<_>>();
        assert_eq!(ids, vec!["chunk-1"]);
    }

    #[test]
    fn test_29_1_2_transform_llm_extractor_parses_and_filter_drops_untrusted_nodes() {
        // SCEN-29.1.2 / AC2 / TEST-29.1.2
        let parsed = parse_llm_extractor_output(
            r#"{"entities":["RAG","Rust","RAG"],"themes":["evaluation"],"summary":"  RAG evaluation in Rust  "}"#,
        )
        .expect("captured extractor output parses");
        let normalized = normalize_extraction_properties(parsed);
        assert_eq!(
            normalized.get("entities"),
            Some(&GraphProperty::TextList(vec![
                "RAG".to_string(),
                "Rust".to_string()
            ]))
        );
        assert_eq!(
            normalized.get("summary"),
            Some(&GraphProperty::Text("RAG evaluation in Rust".to_string()))
        );

        let graph = KnowledgeGraph::new()
            .add_node(
                GraphNode::new("keep", "chunk")
                    .with_property("trusted", GraphProperty::Boolean(true)),
            )
            .add_node(
                GraphNode::new("drop", "chunk")
                    .with_property("trusted", GraphProperty::Boolean(false)),
            )
            .add_edge(GraphEdge::new("keep", "drop", "next"));
        let filtered = filter_graph_by_property(&graph, "trusted", &GraphProperty::Boolean(true));
        assert!(filtered.node("keep").is_some());
        assert!(filtered.node("drop").is_none());
        assert!(filtered.edges.is_empty());
    }

    #[test]
    fn test_29_1_3_pre_chunked_synthesizer_creates_one_sample_per_chunk() {
        // SCEN-29.1.3 / AC3 / TEST-29.1.3
        let persona = PersonaGenerator::new("deterministic-seed").generate(
            "QA Lead",
            "evaluation engineer",
            vec!["verify provided chunks".to_string()],
        );
        let chunks = vec![
            TextChunk::new("chunk-1", "doc-1", 0, "RAG evaluates retrieval."),
            TextChunk::new("chunk-2", "doc-1", 1, "Rust services run fast."),
        ];
        let samples = synthesize_pre_chunked_samples(&chunks, &persona);
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].hop_count, 1);
        assert_eq!(samples[0].source_node_ids, vec!["chunk-1"]);
        assert_eq!(
            samples[0]
                .sample
                .metadata
                .get("synthesis_type")
                .map(String::as_str),
            Some("pre-chunked")
        );
    }

    use async_trait::async_trait;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    /// Mock LLM that replays scripted responses in order and records the prompts it saw,
    /// so the generation pipeline can be driven deterministically without a network call.
    /// Same shape as `metric::tests::ScriptedLlm`.
    struct ScriptedLlm {
        responses: Mutex<VecDeque<String>>,
        prompts: Mutex<Vec<String>>,
    }

    impl ScriptedLlm {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: Mutex::new(responses.into_iter().map(str::to_string).collect()),
                prompts: Mutex::new(Vec::new()),
            }
        }

        fn prompts(&self) -> Vec<String> {
            self.prompts.lock().expect("prompts").clone()
        }
    }

    #[async_trait]
    impl LlmProvider for ScriptedLlm {
        async fn generate(&self, request: LlmRequest) -> Result<crate::LlmResponse, RagasError> {
            let prompt = request
                .messages
                .iter()
                .map(|message| message.content.clone())
                .collect::<Vec<_>>()
                .join("\n");
            self.prompts.lock().expect("prompts").push(prompt);
            let content = self
                .responses
                .lock()
                .expect("responses")
                .pop_front()
                .ok_or_else(|| RagasError::Provider {
                    message: "scripted LLM ran out of responses".to_string(),
                })?;
            Ok(crate::LlmResponse {
                content,
                usage: None,
            })
        }
    }

    const TWO_CHUNK_DOC: &str = "Ragas evaluates retrieval quality with grounded metrics. Rust services compile to fast native binaries.";

    #[tokio::test]
    async fn test_31_2_1_synthesizer_invokes_llm_and_builds_grounded_dataset() {
        // Real-behavior: the LLM is actually called and a well-formed EvaluationDataset
        // (non-empty user_input/response, populated contexts) is produced.
        // Two chunks (max 60 chars) -> 2 single-hop calls + 1 multi-hop call over the
        // single `next` edge = 3 generate() invocations.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"question": "What does Ragas evaluate?", "answer": "Retrieval quality."}"#,
            r#"{"question": "How fast are Rust binaries?", "answer": "They are fast native binaries."}"#,
            r#"{"question": "Do Ragas and Rust both aim for quality and speed?", "answer": "Yes, Ragas evaluates quality and Rust compiles fast."}"#,
        ]));

        let dataset = Synthesizer::new(llm.clone())
            .with_chunk_max_chars(60)
            .generate_testset("doc-1", TWO_CHUNK_DOC)
            .await
            .expect("dataset");

        // The LLM was genuinely invoked once per generated sample, and the prompts carried
        // the real chunk text (proving the pipeline feeds node content to the model).
        let prompts = llm.prompts();
        assert_eq!(prompts.len(), 3);
        assert!(
            prompts
                .iter()
                .all(|prompt| prompt.contains("Ragas") || prompt.contains("Rust"))
        );
        assert!(prompts[2].contains("CONTEXT A") && prompts[2].contains("CONTEXT B"));

        assert_eq!(dataset.len(), 3);
        for sample in dataset.iter() {
            assert!(!sample.user_input.trim().is_empty());
            assert!(!sample.response.trim().is_empty());
            assert!(!sample.retrieved_contexts.is_empty());
            assert!(sample.reference.is_some());
        }

        // The first sample carries the LLM-authored question/answer and the real chunk
        // text as its retrieved context (not a template).
        let single_hop = dataset
            .iter()
            .find(|sample| {
                sample.metadata.get("synthesis_type").map(String::as_str) == Some("single-hop")
            })
            .expect("single-hop sample");
        assert_eq!(single_hop.user_input, "What does Ragas evaluate?");
        assert_eq!(single_hop.response, "Retrieval quality.");
        assert_eq!(single_hop.retrieved_contexts.len(), 1);
        assert!(single_hop.retrieved_contexts[0].contains("Ragas"));

        let multi_hop = dataset
            .iter()
            .find(|sample| {
                sample.metadata.get("synthesis_type").map(String::as_str) == Some("multi-hop")
            })
            .expect("multi-hop sample");
        assert_eq!(multi_hop.retrieved_contexts.len(), 2);
        assert!(multi_hop.user_input.contains("Ragas") || multi_hop.user_input.contains("Rust"));
    }

    #[tokio::test]
    async fn test_31_2_1b_multi_hop_sample_spans_two_chunks_and_prompts_with_both() {
        // First-class multi-hop: with multi-hop enabled, a multi-hop sample must (a) carry
        // BOTH adjacent chunk texts as its retrieved_contexts, (b) be marked multi-hop, and
        // (c) be produced by a real generate() call whose prompt contained BOTH chunk texts.
        //
        // Hand-verified chunking of TWO_CHUNK_DOC at max 60 chars (greedy, whitespace split):
        //   chunk-0 = "Ragas evaluates retrieval quality with grounded metrics." (56 chars)
        //   chunk-1 = "Rust services compile to fast native binaries."
        const CHUNK_0: &str = "Ragas evaluates retrieval quality with grounded metrics.";
        const CHUNK_1: &str = "Rust services compile to fast native binaries.";

        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"question": "What does Ragas evaluate?", "answer": "Retrieval quality."}"#,
            r#"{"question": "How fast are Rust binaries?", "answer": "They are fast native binaries."}"#,
            r#"{"question": "Do Ragas and Rust both target quality and speed?", "answer": "Yes."}"#,
        ]));

        let dataset = Synthesizer::new(llm.clone())
            .with_chunk_max_chars(60)
            .with_multi_hop(true)
            .generate_testset("doc-1", TWO_CHUNK_DOC)
            .await
            .expect("dataset");

        // (a) + (b): exactly one multi-hop sample, and its contexts are the two adjacent
        // chunk texts verbatim (distinct chunks, both present).
        let multi_hop: Vec<_> = dataset
            .iter()
            .filter(|sample| {
                sample.metadata.get("synthesis_type").map(String::as_str) == Some("multi-hop")
            })
            .collect();
        assert_eq!(
            multi_hop.len(),
            1,
            "exactly one adjacent-chunk multi-hop sample"
        );
        let sample = multi_hop[0];
        assert_eq!(
            sample.retrieved_contexts,
            vec![CHUNK_0.to_string(), CHUNK_1.to_string()]
        );
        assert_eq!(
            sample.metadata.get("source_node_ids").map(String::as_str),
            Some("doc-1-chunk-0,doc-1-chunk-1")
        );

        // (c): the LLM prompt that produced the multi-hop sample carried BOTH chunk texts.
        // The multi-hop call is the third generate() invocation (after the two single-hop
        // calls), and it is the only prompt mentioning both chunks via CONTEXT A / CONTEXT B.
        let prompts = llm.prompts();
        assert_eq!(prompts.len(), 3);
        let multi_hop_prompt = prompts
            .iter()
            .find(|prompt| prompt.contains("CONTEXT A") && prompt.contains("CONTEXT B"))
            .expect("a multi-hop prompt with both contexts");
        assert!(
            multi_hop_prompt.contains(CHUNK_0),
            "multi-hop prompt missing chunk-0 text: {multi_hop_prompt}"
        );
        assert!(
            multi_hop_prompt.contains(CHUNK_1),
            "multi-hop prompt missing chunk-1 text: {multi_hop_prompt}"
        );

        // The single-hop prompts each carry exactly one chunk, never both -> the only place
        // both chunks meet is the genuine multi-hop call.
        let both_chunk_prompts = prompts
            .iter()
            .filter(|prompt| prompt.contains(CHUNK_0) && prompt.contains(CHUNK_1))
            .count();
        assert_eq!(both_chunk_prompts, 1);
    }

    #[tokio::test]
    async fn test_31_2_2_synthesizer_single_hop_only_skips_multi_hop_calls() {
        // With multi-hop disabled, only the single-hop calls are made (one per chunk).
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"question": "Q1?", "answer": "A1."}"#,
            r#"{"question": "Q2?", "answer": "A2."}"#,
        ]));

        let dataset = Synthesizer::new(llm.clone())
            .with_chunk_max_chars(60)
            .with_multi_hop(false)
            .generate_testset("doc-1", TWO_CHUNK_DOC)
            .await
            .expect("dataset");

        // Only the single-hop calls fire (one per chunk); no multi-hop generate() over the
        // `next` edge, so no prompt ever carries both chunks at once.
        let prompts = llm.prompts();
        assert_eq!(prompts.len(), 2);
        assert!(
            prompts
                .iter()
                .all(|prompt| { !(prompt.contains("CONTEXT A") && prompt.contains("CONTEXT B")) })
        );
        assert_eq!(dataset.len(), 2);
        assert!(dataset.iter().all(|sample| {
            sample.metadata.get("synthesis_type").map(String::as_str) == Some("single-hop")
        }));
        // The defining contrast vs. multi-hop enabled: zero multi-hop samples are produced.
        assert_eq!(
            dataset
                .iter()
                .filter(|sample| {
                    sample.metadata.get("synthesis_type").map(String::as_str) == Some("multi-hop")
                })
                .count(),
            0
        );
    }

    #[tokio::test]
    async fn test_31_2_3_empty_document_returns_empty_dataset_error() {
        // Adversarial: a document with no usable text yields EmptyDataset and never calls
        // the LLM.
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let error = generate_testset(llm.clone(), "doc-1", "   \n  \t ")
            .await
            .expect_err("empty document should error");
        assert_eq!(error, RagasError::EmptyDataset);
        assert!(llm.prompts().is_empty());
    }

    #[tokio::test]
    async fn test_31_2_4_malformed_llm_json_propagates_parse_error() {
        // Adversarial: a non-JSON LLM reply surfaces as a parse error rather than a fake
        // sample.
        let llm = Arc::new(ScriptedLlm::new(vec!["this is not json at all"]));
        let error = generate_testset(llm, "doc-1", "Ragas evaluates retrieval.")
            .await
            .expect_err("malformed JSON should error");
        match error {
            RagasError::Parse { message } => {
                assert!(message.contains("single-hop testset synthesis"));
            }
            other => panic!("expected parse error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_31_2_5_json_repair_path_recovers_fenced_output() {
        // The repair path extracts the outermost { .. } block from prose/markdown fences,
        // mirroring metric::extract_json_block.
        let llm = Arc::new(ScriptedLlm::new(vec![
            "Sure! Here is the data:\n```json\n{\"question\": \"What is RAG?\", \"answer\": \"Retrieval augmented generation.\"}\n```\nHope that helps.",
        ]));
        let dataset = generate_testset(llm, "doc-1", "RAG augments generation with retrieval.")
            .await
            .expect("repaired dataset");
        assert_eq!(dataset.len(), 1);
        assert_eq!(dataset.iter().next().unwrap().user_input, "What is RAG?");
        assert_eq!(
            dataset.iter().next().unwrap().response,
            "Retrieval augmented generation."
        );
    }

    #[tokio::test]
    async fn test_31_2_6_empty_qa_fields_are_rejected() {
        // A structurally-valid JSON object with blank fields must not become a sample.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"question": "  ", "answer": ""}"#,
        ]));
        let error = generate_testset(llm, "doc-1", "Ragas evaluates retrieval.")
            .await
            .expect_err("blank fields should error");
        assert!(matches!(error, RagasError::Parse { .. }));
    }

    /// Live test against the real OpenAI-compatible model. Ignored by default and skipped
    /// (returns early) unless OPENAI_API_KEY is set. Math/structure is verified via mocks
    /// above; this path is real-LLM UNVERIFIED until run with a live key.
    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; run with --ignored"]
    async fn test_31_2_7_live_synthesizer_generates_from_real_model() {
        let Some(client) = crate::ProviderConfig::from_env().chat_client() else {
            eprintln!("skipping live testset synthesis: OPENAI_API_KEY not set");
            return;
        };
        let llm: Arc<dyn LlmProvider> = Arc::new(client);
        let document = "The Ragas library evaluates retrieval-augmented generation systems. \
It provides metrics such as faithfulness and context precision to score answers \
against retrieved evidence.";
        let dataset = Synthesizer::new(llm)
            .with_multi_hop(false)
            .generate_testset("live-doc", document)
            .await
            .expect("live dataset");

        assert!(!dataset.is_empty());
        for sample in dataset.iter() {
            assert!(!sample.user_input.trim().is_empty());
            assert!(!sample.response.trim().is_empty());
            assert!(!sample.retrieved_contexts.is_empty());
        }
    }

    /// Live multi-hop test against the real model. Ignored/env-gated like the single-hop
    /// live test. Multi-hop structure is verified via mocks above; the real-LLM path is
    /// UNVERIFIED until run with a live key. Reads OPENAI_MODEL (never hardcoded).
    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; run with --ignored"]
    async fn test_31_2_8_live_multi_hop_synthesizer_generates_two_context_sample() {
        let Some(client) = crate::ProviderConfig::from_env().chat_client() else {
            eprintln!("skipping live multi-hop testset synthesis: OPENAI_API_KEY not set");
            return;
        };
        let llm: Arc<dyn LlmProvider> = Arc::new(client);
        let document = "The Ragas library evaluates retrieval-augmented generation systems. \
It provides metrics such as faithfulness and context precision to score answers \
against retrieved evidence. Rust services compile these metrics into fast native \
binaries that run without a Python runtime.";
        // Small chunks force >= 2 chunks so an adjacent `next` edge exists for multi-hop.
        let dataset = Synthesizer::new(llm)
            .with_chunk_max_chars(120)
            .with_multi_hop(true)
            .generate_testset("live-multi-doc", document)
            .await
            .expect("live multi-hop dataset");

        let multi_hop: Vec<_> = dataset
            .iter()
            .filter(|sample| {
                sample.metadata.get("synthesis_type").map(String::as_str) == Some("multi-hop")
            })
            .collect();
        assert!(
            !multi_hop.is_empty(),
            "expected at least one multi-hop sample from the real model"
        );
        for sample in &multi_hop {
            assert_eq!(sample.retrieved_contexts.len(), 2);
            assert!(!sample.user_input.trim().is_empty());
            assert!(!sample.response.trim().is_empty());
        }
    }

    fn text_node(id: &str, text: &str) -> GraphNode {
        GraphNode::new(id, "chunk").with_property("text", GraphProperty::Text(text.to_string()))
    }

    #[tokio::test]
    async fn llm_extractor_ner_parses_entity_list() {
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"entities": ["Elon Musk", "Tesla", "SpaceX"]}"#,
        ]));
        let node = text_node("n1", "Elon Musk runs Tesla and SpaceX.");
        let (name, property) = LlmExtractor::new(llm.clone(), LlmExtractorKind::Ner)
            .extract(&node)
            .await
            .expect("ner");
        assert_eq!(name, "entities");
        assert_eq!(
            property,
            GraphProperty::TextList(vec![
                "Elon Musk".to_string(),
                "Tesla".to_string(),
                "SpaceX".to_string(),
            ])
        );
        // The model saw the node text and was asked for the `entities` JSON shape.
        let prompt = &llm.prompts()[0];
        assert!(prompt.contains("Elon Musk runs Tesla"));
        assert!(prompt.contains("\"entities\""));
    }

    #[tokio::test]
    async fn llm_extractor_summary_parses_single_text() {
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"text": "A concise summary."}"#]));
        let node = text_node("n1", "A long passage about many different things.");
        let (name, property) = LlmExtractor::new(llm, LlmExtractorKind::Summary)
            .extract(&node)
            .await
            .expect("summary");
        assert_eq!(name, "summary");
        assert_eq!(
            property,
            GraphProperty::Text("A concise summary.".to_string())
        );
    }

    #[tokio::test]
    async fn llm_extractor_recovers_fenced_json() {
        // The repair path extracts the outermost { .. } block from prose/markdown fences.
        let llm = Arc::new(ScriptedLlm::new(vec![
            "Sure!\n```json\n{\"output\": [\"AI\", \"Automation\"]}\n```\n",
        ]));
        let node = text_node("n1", "AI automates tasks.");
        let (name, property) = LlmExtractor::new(llm, LlmExtractorKind::Themes)
            .extract(&node)
            .await
            .expect("themes");
        assert_eq!(name, "themes");
        assert_eq!(
            property,
            GraphProperty::TextList(vec!["AI".to_string(), "Automation".to_string()])
        );
    }

    #[tokio::test]
    async fn llm_extractor_list_extends_across_chunks() {
        // A small char budget forces two chunks -> two generate calls -> concatenated list,
        // mirroring Python's per-chunk `extend` loop.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"entities": ["Alpha"]}"#,
            r#"{"entities": ["Beta", "Gamma"]}"#,
        ]));
        let node = text_node("n1", "Alpha one two three. Beta four five six.");
        let (_, property) = LlmExtractor::new(llm.clone(), LlmExtractorKind::Ner)
            .with_max_chars(20)
            .extract(&node)
            .await
            .expect("entities");
        assert_eq!(
            property,
            GraphProperty::TextList(vec![
                "Alpha".to_string(),
                "Beta".to_string(),
                "Gamma".to_string(),
            ])
        );
        assert_eq!(llm.prompts().len(), 2);
    }

    #[tokio::test]
    async fn llm_extractor_missing_text_returns_empty_without_calling_model() {
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let node = GraphNode::new("n1", "chunk"); // no `text` property

        let (name, property) = LlmExtractor::new(llm.clone(), LlmExtractorKind::Ner)
            .extract(&node)
            .await
            .expect("empty list");
        assert_eq!(name, "entities");
        assert_eq!(property, GraphProperty::TextList(Vec::new()));

        let (_, property) = LlmExtractor::new(llm.clone(), LlmExtractorKind::Summary)
            .extract(&node)
            .await
            .expect("empty text");
        assert_eq!(property, GraphProperty::Text(String::new()));

        // Lenient empty path never touches the model (matching Python's early `None`/`[]`).
        assert!(llm.prompts().is_empty());
    }

    #[tokio::test]
    async fn llm_extractor_malformed_json_errors() {
        let llm = Arc::new(ScriptedLlm::new(vec!["not json at all"]));
        let node = text_node("n1", "Some text.");
        let result = LlmExtractor::new(llm, LlmExtractorKind::Ner)
            .extract(&node)
            .await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn llm_extractor_wrong_shape_errors() {
        // Valid JSON but the expected field is absent -> a typed parse error, not a silent empty.
        // (Covers the list-kind extract_list_chunk missing-field branch via Ner.)
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"wrong_key": ["x"]}"#]));
        let node = text_node("n1", "Some text.");
        let result = LlmExtractor::new(llm, LlmExtractorKind::Ner)
            .extract(&node)
            .await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn llm_extractor_single_value_wrong_shape_errors() {
        // The single-value parse path (extract_single_chunk) also rejects a missing field —
        // the other half of the wrong-shape branch from the list-kind test above.
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"not_text": "x"}"#]));
        let node = text_node("n1", "Some text.");
        let result = LlmExtractor::new(llm, LlmExtractorKind::Summary)
            .extract(&node)
            .await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn llm_extractor_all_kinds_wire_property_key_and_shape() {
        // Pin the per-kind config table (property_name / json_key / list-vs-single) against
        // copy-paste drift: for each of the 7 kinds, a response in that kind's JSON shape must
        // yield the right property name + value, and the prompt must name the kind's JSON key.
        struct Case {
            kind: LlmExtractorKind,
            response: &'static str,
            property: &'static str,
            json_key: &'static str,
            expected: GraphProperty,
        }
        let cases = vec![
            Case {
                kind: LlmExtractorKind::Summary,
                response: r#"{"text": "S"}"#,
                property: "summary",
                json_key: "\"text\"",
                expected: GraphProperty::Text("S".to_string()),
            },
            Case {
                kind: LlmExtractorKind::Title,
                response: r#"{"text": "T"}"#,
                property: "title",
                json_key: "\"text\"",
                expected: GraphProperty::Text("T".to_string()),
            },
            Case {
                kind: LlmExtractorKind::TopicDescription,
                response: r#"{"description": "D"}"#,
                property: "topic_description",
                json_key: "\"description\"",
                expected: GraphProperty::Text("D".to_string()),
            },
            Case {
                kind: LlmExtractorKind::Keyphrases,
                response: r#"{"keyphrases": ["k1", "k2"]}"#,
                property: "keyphrases",
                json_key: "\"keyphrases\"",
                expected: GraphProperty::TextList(vec!["k1".to_string(), "k2".to_string()]),
            },
            Case {
                kind: LlmExtractorKind::Headlines,
                response: r#"{"headlines": ["h1"]}"#,
                property: "headlines",
                json_key: "\"headlines\"",
                expected: GraphProperty::TextList(vec!["h1".to_string()]),
            },
            Case {
                kind: LlmExtractorKind::Ner,
                response: r#"{"entities": ["e1"]}"#,
                property: "entities",
                json_key: "\"entities\"",
                expected: GraphProperty::TextList(vec!["e1".to_string()]),
            },
            Case {
                kind: LlmExtractorKind::Themes,
                response: r#"{"output": ["t1"]}"#,
                property: "themes",
                json_key: "\"output\"",
                expected: GraphProperty::TextList(vec!["t1".to_string()]),
            },
        ];

        for case in cases {
            let llm = Arc::new(ScriptedLlm::new(vec![case.response]));
            let extractor = LlmExtractor::new(llm.clone(), case.kind);
            assert_eq!(
                extractor.property_name(),
                case.property,
                "kind {:?}",
                case.kind
            );
            let (name, property) = extractor
                .extract(&text_node("n", "Some node text."))
                .await
                .unwrap_or_else(|error| panic!("kind {:?}: {error}", case.kind));
            assert_eq!(name, case.property, "kind {:?}", case.kind);
            assert_eq!(property, case.expected, "kind {:?}", case.kind);
            assert!(
                llm.prompts()[0].contains(case.json_key),
                "kind {:?} prompt missing key {}",
                case.kind,
                case.json_key
            );
        }
    }

    #[tokio::test]
    async fn llm_extractor_with_max_num_changes_prompt() {
        // with_max_num overrides the prompt cap for list kinds (default Keyphrases = 5).
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"keyphrases": ["k"]}"#]));
        LlmExtractor::new(llm.clone(), LlmExtractorKind::Keyphrases)
            .with_max_num(17)
            .extract(&text_node("n", "Some text."))
            .await
            .expect("keyphrases");
        assert!(
            llm.prompts()[0].contains("top 17 keyphrases"),
            "prompt should reflect the overridden max_num, got: {}",
            llm.prompts()[0]
        );
    }

    #[tokio::test]
    async fn extract_bundle_assembles_entities_themes_summary() {
        // Fixed sequential order: NER, then Themes, then Summary.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"entities": ["RAG", "retrieval"]}"#,
            r#"{"output": ["evaluation", "grounding"]}"#,
            r#"{"text": "RAG evaluates retrieval with grounded metrics."}"#,
        ]));
        let node = text_node(
            "n1",
            "RAG evaluates retrieval quality with grounded metrics.",
        );
        let bundle = extract_bundle(llm.clone(), &node).await.expect("bundle");
        assert_eq!(
            bundle.entities,
            vec!["RAG".to_string(), "retrieval".to_string()]
        );
        assert_eq!(
            bundle.themes,
            vec!["evaluation".to_string(), "grounding".to_string()]
        );
        assert_eq!(
            bundle.summary,
            "RAG evaluates retrieval with grounded metrics."
        );
        assert_eq!(llm.prompts().len(), 3);

        // The bundle feeds the existing attach_extractions substrate end-to-end.
        let updated = attach_extractions(KnowledgeGraph::new().add_node(node), "n1", bundle);
        let stored = updated.node("n1").expect("node");
        let entities = match stored.properties.get("entities") {
            Some(GraphProperty::TextList(values)) => values.clone(),
            other => panic!("expected entities list, got {other:?}"),
        };
        assert!(
            entities.contains(&"RAG".to_string()) && entities.contains(&"retrieval".to_string())
        );
        assert!(matches!(
            stored.properties.get("summary"),
            Some(GraphProperty::Text(summary)) if summary.contains("RAG evaluates")
        ));
    }

    /// Live extraction gate (env-gated like the synthesizer live tests): the real model pulls
    /// the obvious named entities out of an entity-rich passage and returns a non-empty summary
    /// shorter than the source. This is the "real LLM, real extraction" proof for the new
    /// `LlmExtractor`.
    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; run with --ignored"]
    async fn live_llm_extractor_pulls_named_entities_and_summary() {
        let Some(client) = crate::ProviderConfig::from_env().chat_client() else {
            eprintln!("skipping live extractor: OPENAI_API_KEY not set");
            return;
        };
        let llm: Arc<dyn LlmProvider> = Arc::new(client);
        let text = "Elon Musk, the CEO of Tesla and SpaceX, announced plans to expand \
operations to new locations in Europe and Asia, creating thousands of jobs in cities \
like Berlin and Shanghai.";
        let node = text_node("live-node", text);

        let (name, entities) = LlmExtractor::new(llm.clone(), LlmExtractorKind::Ner)
            .extract(&node)
            .await
            .expect("live ner");
        assert_eq!(name, "entities");
        let GraphProperty::TextList(entities) = entities else {
            panic!("expected an entity list");
        };
        assert!(
            !entities.is_empty(),
            "expected some entities from the real model"
        );
        let lower: Vec<String> = entities.iter().map(|item| item.to_lowercase()).collect();
        assert!(
            lower.iter().any(|item| item.contains("tesla"))
                && lower
                    .iter()
                    .any(|item| item.contains("musk") || item.contains("spacex")),
            "expected Tesla + Musk/SpaceX among entities, got {entities:?}"
        );

        let (_, summary) = LlmExtractor::new(llm, LlmExtractorKind::Summary)
            .extract(&node)
            .await
            .expect("live summary");
        let GraphProperty::Text(summary) = summary else {
            panic!("expected summary text");
        };
        assert!(!summary.trim().is_empty(), "expected a non-empty summary");
        assert!(
            summary.len() < text.len(),
            "summary ({} chars) should be shorter than source ({} chars)",
            summary.len(),
            text.len()
        );
    }

    #[test]
    fn graph_property_vector_roundtrips() {
        let graph = KnowledgeGraph::new().add_node(
            GraphNode::new("n", "chunk")
                .with_property("embedding", GraphProperty::Vector(vec![0.1, 0.2, 0.3])),
        );
        let json = serde_json::to_string(&graph).expect("serialize");
        let back: KnowledgeGraph = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, graph);
        assert!(matches!(
            back.node("n").unwrap().properties.get("embedding"),
            Some(GraphProperty::Vector(values)) if values.len() == 3
        ));
    }

    fn embedded_node(id: &str, vector: Vec<f32>) -> GraphNode {
        GraphNode::new(id, "chunk").with_property("embedding", GraphProperty::Vector(vector))
    }

    #[test]
    fn cosine_builder_links_similar_above_threshold() {
        // a == b (cosine 1.0); c is orthogonal to both (cosine 0.0).
        let graph = KnowledgeGraph::new()
            .add_node(embedded_node("a", vec![1.0, 0.0]))
            .add_node(embedded_node("b", vec![1.0, 0.0]))
            .add_node(embedded_node("c", vec![0.0, 1.0]));

        let linked = build_cosine_relationships(graph, 0.5).expect("build");
        let edges = linked.edges_by_relationship("cosine_similarity");
        // Only a<->b clears the 0.5 threshold; a-c and b-c are 0.0.
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].source_id, "a");
        assert_eq!(edges[0].target_id, "b");
        let Some(GraphProperty::Number(score)) = edges[0].properties.get("cosine_similarity")
        else {
            panic!("expected a numeric cosine_similarity score");
        };
        assert!(
            *score > 0.99,
            "identical vectors should score ~1.0, got {score}"
        );
    }

    #[test]
    fn cosine_builder_skips_nodes_without_embedding() {
        // A non-embedded node is skipped (not an error) — the documented divergence from
        // Python, which pre-filters via its transforms engine.
        let graph = KnowledgeGraph::new()
            .add_node(
                GraphNode::new("doc", "document")
                    .with_property("title", GraphProperty::Text("T".to_string())),
            )
            .add_node(embedded_node("a", vec![1.0, 0.0]))
            .add_node(embedded_node("b", vec![1.0, 0.0]));
        let linked = build_cosine_relationships(graph, 0.5).expect("build");
        assert_eq!(linked.edges_by_relationship("cosine_similarity").len(), 1);
    }

    #[test]
    fn cosine_builder_rejects_mismatched_dimensions() {
        let graph = KnowledgeGraph::new()
            .add_node(embedded_node("a", vec![1.0, 0.0]))
            .add_node(embedded_node("b", vec![1.0, 0.0, 0.0]));
        let Err(RagasError::Parse { message }) = build_cosine_relationships(graph, 0.5) else {
            panic!("expected a Parse error on mismatched embedding dimensions");
        };
        // The error names the offending node and the dimensions, for actionable diagnosis.
        assert!(
            message.contains('b') && message.contains("length") && message.contains("expected"),
            "error should identify node + dimensions, got: {message}"
        );
    }

    #[test]
    fn cosine_builder_empty_and_single_node_return_no_edges() {
        let empty = build_cosine_relationships(KnowledgeGraph::new(), 0.5).expect("empty graph");
        assert!(empty.edges_by_relationship("cosine_similarity").is_empty());

        // A single embedded node has no i<j pair, so no edges (but the node is preserved).
        let single = build_cosine_relationships(
            KnowledgeGraph::new().add_node(embedded_node("only", vec![1.0, 0.0])),
            0.5,
        )
        .expect("single node");
        assert_eq!(single.nodes.len(), 1);
        assert!(single.edges_by_relationship("cosine_similarity").is_empty());
    }

    #[test]
    fn cosine_builder_threshold_is_inclusive_and_filters_negative() {
        // x⊥y (cosine 0.0), x·z anti-parallel (cosine -1.0), y⊥z (cosine 0.0).
        let graph = || {
            KnowledgeGraph::new()
                .add_node(embedded_node("x", vec![1.0, 0.0]))
                .add_node(embedded_node("y", vec![0.0, 1.0]))
                .add_node(embedded_node("z", vec![-1.0, 0.0]))
        };
        // threshold 0.0 is inclusive (>=): the two orthogonal (0.0) pairs link; the -1.0 pair
        // is filtered out.
        let at_zero = build_cosine_relationships(graph(), 0.0).expect("build");
        let edges = at_zero.edges_by_relationship("cosine_similarity");
        assert_eq!(
            edges.len(),
            2,
            "0.0-threshold includes the two 0.0 pairs only"
        );
        assert!(
            !edges
                .iter()
                .any(|edge| edge.source_id == "x" && edge.target_id == "z"),
            "the anti-parallel x-z pair (-1.0) must be filtered at threshold 0.0"
        );
        // Lowering the threshold to -1.0 admits the anti-parallel pair too (all 3 pairs).
        let at_neg = build_cosine_relationships(graph(), -1.0).expect("build");
        assert_eq!(at_neg.edges_by_relationship("cosine_similarity").len(), 3);
    }

    #[tokio::test]
    async fn embedding_extractor_stores_vector_and_errors_on_missing_text() {
        let embedding = Arc::new(crate::MockEmbeddingProvider::new(vec![vec![0.1, 0.2, 0.3]]));
        let (name, property) = EmbeddingExtractor::new(embedding.clone())
            .extract(&text_node("n1", "some text to embed"))
            .await
            .expect("embedding");
        assert_eq!(name, "embedding");
        assert_eq!(property, GraphProperty::Vector(vec![0.1, 0.2, 0.3]));

        // Missing text property -> error (faithful to Python's ValueError, unlike LlmExtractor).
        let result = EmbeddingExtractor::new(embedding)
            .extract(&GraphNode::new("n2", "chunk"))
            .await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn embedding_extractor_property_name_overrides() {
        let embedding = Arc::new(crate::MockEmbeddingProvider::new(vec![vec![0.5, 0.5]]));
        // Text lives under "body" (not the default "text"); the embedding goes to "vec".
        let node = GraphNode::new("n", "chunk").with_property(
            "body",
            GraphProperty::Text("text under a custom key".to_string()),
        );
        let (name, property) = EmbeddingExtractor::new(embedding)
            .with_property_name("vec")
            .with_embed_property_name("body")
            .extract(&node)
            .await
            .expect("embedding");
        // Reading from "body" succeeded (the node has no "text") and the output key is "vec".
        assert_eq!(name, "vec");
        assert_eq!(property, GraphProperty::Vector(vec![0.5, 0.5]));
    }

    /// Live gate (env-gated): real embeddings make two semantically similar nodes score
    /// strictly higher than an unrelated node. Threshold -1.0 forces an edge per pair so the
    /// scores are comparable; the proof is the ordering, not an absolute cutoff.
    #[tokio::test]
    #[ignore = "requires embedding provider env; run with --ignored"]
    async fn live_cosine_relationships_link_semantically_similar_nodes() {
        let Some(client) = crate::ProviderConfig::from_env().embedding_client() else {
            eprintln!("skipping live cosine builder: embedding provider not set");
            return;
        };
        let embedding: Arc<dyn EmbeddingProvider> = Arc::new(client);

        let texts = [
            (
                "cat-1",
                "Cats are small domestic felines commonly kept as pets.",
            ),
            ("cat-2", "Domestic cats are popular household pet animals."),
            (
                "market",
                "The stock market fell sharply after the interest rate decision.",
            ),
        ];
        let mut graph = KnowledgeGraph::new();
        for (id, text) in texts {
            let (name, vector) = EmbeddingExtractor::new(embedding.clone())
                .extract(&text_node(id, text))
                .await
                .expect("live embed");
            graph = graph.add_node(text_node(id, text).with_property(name, vector));
        }

        let linked = build_cosine_relationships(graph, -1.0).expect("build");
        let edges = linked.edges_by_relationship("cosine_similarity");
        let score = |a: &str, b: &str| -> f64 {
            edges
                .iter()
                .find(|edge| {
                    (edge.source_id == a && edge.target_id == b)
                        || (edge.source_id == b && edge.target_id == a)
                })
                .and_then(|edge| match edge.properties.get("cosine_similarity") {
                    Some(GraphProperty::Number(value)) => Some(*value),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("no cosine edge for {a},{b}"))
        };

        let cats = score("cat-1", "cat-2");
        let cat_market = score("cat-1", "market");
        // Stronger than bare ordering: the two cat sentences must be substantially similar in
        // absolute terms AND beat the unrelated market sentence by a clear margin — so the gate
        // can't pass on degenerate/collapsed embeddings.
        assert!(
            cats > 0.5,
            "two clearly-related cat sentences should be substantially similar, got {cats}"
        );
        assert!(
            cats - cat_market > 0.1,
            "cat/cat similarity ({cats}) should exceed cat/market ({cat_market}) by a clear margin"
        );
    }

    fn entitied_node(id: &str, entities: &[&str]) -> GraphNode {
        GraphNode::new(id, "chunk").with_property(
            "entities",
            GraphProperty::TextList(entities.iter().map(|e| e.to_string()).collect()),
        )
    }

    fn overlap_score(graph: &KnowledgeGraph, source: &str, target: &str) -> Option<f64> {
        graph
            .edges_by_relationship("entities_overlap")
            .into_iter()
            .find(|edge| edge.source_id == source && edge.target_id == target)
            .and_then(|edge| match edge.properties.get("entities_overlap_score") {
                Some(GraphProperty::Number(value)) => Some(*value),
                _ => None,
            })
    }

    #[test]
    fn overlap_builder_links_nodes_sharing_entities() {
        // "zzz" appears in all 3 nodes -> it is the single noisy item (top 5%, max(1)),
        // so it is excluded from scoring; the shared "Tesla" drives the only overlap.
        let graph = KnowledgeGraph::new()
            .add_node(entitied_node("n1", &["zzz", "Tesla", "SpaceX"]))
            .add_node(entitied_node("n2", &["zzz", "Tesla", "Berlin"]))
            .add_node(entitied_node("n3", &["zzz", "Apple", "Google"]));

        let linked = build_overlap_relationships(graph, 0.9, 0.01);
        let edges = linked.edges_by_relationship("entities_overlap");
        // Only n1->n2 shares an entity (Tesla); n*-n3 share nothing.
        assert_eq!(edges.len(), 1);
        assert_eq!(
            (edges[0].source_id.as_str(), edges[0].target_id.as_str()),
            ("n1", "n2")
        );
        // 1 match (Tesla=Tesla) of 4 non-noisy comparisons ([Tesla,SpaceX] x [Tesla,Berlin]).
        // If "zzz" were NOT excluded the score would be 2/9 ≈ 0.222, not 0.25 — so this pins
        // the noisy-item exclusion.
        let score = overlap_score(&linked, "n1", "n2").expect("score");
        assert!(
            (score - 0.25).abs() < 1e-9,
            "expected 1/4 = 0.25, got {score}"
        );
        let Some(GraphProperty::TextList(items)) = edges[0].properties.get("overlapped_items")
        else {
            panic!("expected overlapped_items list");
        };
        assert_eq!(items, &vec!["Tesla => Tesla".to_string()]);
        // The relationship is directed: there is no reverse n2->n1 edge.
        assert_eq!(overlap_score(&linked, "n2", "n1"), None);
    }

    #[test]
    fn overlap_builder_excludes_the_most_common_noisy_entity() {
        // "common" is in every node and is the single noisy item; once excluded no two nodes
        // share a (fuzzily) matching entity, so NO edges are produced. Without the exclusion,
        // common=common would link every pair.
        let graph = KnowledgeGraph::new()
            .add_node(entitied_node("n1", &["common", "Tesla"]))
            .add_node(entitied_node("n2", &["common", "Apple"]))
            .add_node(entitied_node("n3", &["common", "Google"]))
            .add_node(entitied_node("n4", &["common", "Berlin"]));

        let linked = build_overlap_relationships(graph, 0.9, 0.01);
        assert!(
            linked.edges_by_relationship("entities_overlap").is_empty(),
            "the only shared entity was noisy and excluded -> no overlap edges"
        );
    }

    #[test]
    fn overlap_builder_matches_fuzzy_near_duplicates() {
        // "zzz" is noisy/excluded; "Microsoft" vs "Microsft" (one-char typo) must clear the
        // 0.9 Jaro-Winkler bar, proving the match is fuzzy, not exact.
        let graph = KnowledgeGraph::new()
            .add_node(entitied_node("n1", &["zzz", "Microsoft"]))
            .add_node(entitied_node("n2", &["zzz", "Microsft"]));

        let linked = build_overlap_relationships(graph, 0.9, 0.01);
        let score = overlap_score(&linked, "n1", "n2");
        assert_eq!(
            score,
            Some(1.0),
            "the single non-noisy comparison (Microsoft~Microsft) should match -> 1/1"
        );
        // Directly pin the underlying fuzzy similarity: the typo pair clears 0.9 but is < 1.0
        // (not an exact match), so the edge is genuinely from fuzzy matching.
        let jw =
            string_distance_similarity_with("microsoft", "microsft", DistanceMeasure::JaroWinkler)
                .score
                .expect("jaro-winkler score");
        assert!(
            (0.9..1.0).contains(&jw),
            "expected 0.9 <= JW < 1.0, got {jw}"
        );
    }

    #[test]
    fn overlap_builder_score_threshold_is_inclusive() {
        // "zzz" is the single noisy item (tie at count 2, first-seen); non-noisy entities are
        // [Tesla, Foo] x [Tesla, Bar] -> 1 match / 4 comparisons = exactly 0.25.
        let graph = || {
            KnowledgeGraph::new()
                .add_node(entitied_node("n1", &["zzz", "Tesla", "Foo"]))
                .add_node(entitied_node("n2", &["zzz", "Tesla", "Bar"]))
        };
        // score (0.25) >= threshold (0.25) is inclusive -> edge.
        let at = build_overlap_relationships(graph(), 0.9, 0.25);
        assert_eq!(overlap_score(&at, "n1", "n2"), Some(0.25));
        // Just above the score -> filtered out.
        let above = build_overlap_relationships(graph(), 0.9, 0.26);
        assert_eq!(overlap_score(&above, "n1", "n2"), None);
    }

    #[test]
    fn overlap_builder_records_multiple_overlaps_in_iteration_order() {
        // All three entities tie at count 2; "zzz" (first-seen) is the noisy item. The non-noisy
        // [Apple, Google] x [Apple, Google] yields two matches (2/4 = 0.5), listed in
        // outer-then-inner order.
        let graph = KnowledgeGraph::new()
            .add_node(entitied_node("n1", &["zzz", "Apple", "Google"]))
            .add_node(entitied_node("n2", &["zzz", "Apple", "Google"]));
        let linked = build_overlap_relationships(graph, 0.9, 0.01);
        assert_eq!(overlap_score(&linked, "n1", "n2"), Some(0.5));
        let edge = linked.edges_by_relationship("entities_overlap")[0];
        let Some(GraphProperty::TextList(items)) = edge.properties.get("overlapped_items") else {
            panic!("expected overlapped_items list");
        };
        assert_eq!(
            items,
            &vec!["Apple => Apple".to_string(), "Google => Google".to_string()]
        );
    }

    #[test]
    fn overlap_builder_skips_nodes_without_entities() {
        // A node lacking `entities` is skipped (the documented divergence), not an error.
        let graph = KnowledgeGraph::new()
            .add_node(
                GraphNode::new("doc", "document")
                    .with_property("title", GraphProperty::Text("T".to_string())),
            )
            .add_node(entitied_node("n1", &["zzz", "Tesla"]))
            .add_node(entitied_node("n2", &["zzz", "Tesla"]));

        let linked = build_overlap_relationships(graph, 0.9, 0.01);
        // "zzz" is the noisy item (tie with Tesla on count 2, first-seen wins); Tesla then
        // matches across n1/n2 -> exactly one edge, and the doc node caused no panic.
        let edges = linked.edges_by_relationship("entities_overlap");
        assert_eq!(edges.len(), 1);
        assert_eq!(
            (edges[0].source_id.as_str(), edges[0].target_id.as_str()),
            ("n1", "n2")
        );
    }

    /// A doc node (with a `summary`) and two chunks linked by `contains`, the layout
    /// `CustomNodeFilter` scores against.
    fn doc_with_chunks(summary: &str, chunks: &[(&str, &str)]) -> KnowledgeGraph {
        let mut graph = KnowledgeGraph::new().add_node(
            GraphNode::new("doc", "document")
                .with_property("summary", GraphProperty::Text(summary.to_string())),
        );
        for (id, text) in chunks {
            graph = graph
                .add_node(text_node(id, text))
                .add_edge(GraphEdge::new("doc", *id, "contains"));
        }
        graph
    }

    #[tokio::test]
    async fn custom_node_filter_removes_low_scoring_chunks() {
        // Scored in node order: keep-chunk (5) kept, drop-chunk (1 <= min_score 2) removed.
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"score": 5}"#, r#"{"score": 1}"#]));
        let graph = doc_with_chunks(
            "A guide to RAG evaluation.",
            &[
                ("keep", "RAG evaluation metrics."),
                ("drop", "unrelated noise"),
            ],
        );
        let filtered = CustomNodeFilter::new(llm)
            .filter(graph)
            .await
            .expect("filter");

        let ids: Vec<&str> = filtered.nodes.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"doc") && ids.contains(&"keep"));
        assert!(
            !ids.contains(&"drop"),
            "the low-scoring chunk should be removed"
        );
        // The removed chunk's incident `contains` edge is gone too.
        assert!(
            !filtered
                .edges
                .iter()
                .any(|edge| edge.source_id == "drop" || edge.target_id == "drop")
        );
        assert_eq!(filtered.edges_by_relationship("contains").len(), 1);
    }

    #[tokio::test]
    async fn custom_node_filter_keeps_unscoreable_chunks_without_calling_model() {
        // Three chunks that must be KEPT and never scored: parent with no summary, parent with
        // an empty summary, and an orphaned chunk with no `contains` edge at all.
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let graph = KnowledgeGraph::new()
            .add_node(GraphNode::new("doc_none", "document"))
            .add_node(
                GraphNode::new("doc_empty", "document")
                    .with_property("summary", GraphProperty::Text(String::new())),
            )
            .add_node(text_node("no_summary", "text a"))
            .add_node(text_node("empty_summary", "text b"))
            .add_node(text_node("orphan", "text c"))
            .add_edge(GraphEdge::new("doc_none", "no_summary", "contains"))
            .add_edge(GraphEdge::new("doc_empty", "empty_summary", "contains"));

        let filtered = CustomNodeFilter::new(llm.clone())
            .filter(graph)
            .await
            .expect("filter");

        for id in ["no_summary", "empty_summary", "orphan"] {
            assert!(filtered.node(id).is_some(), "{id} should be kept");
        }
        assert!(
            llm.prompts().is_empty(),
            "unscoreable chunks must not trigger a scoring call"
        );
    }

    #[tokio::test]
    async fn custom_node_filter_removes_multiple_and_keeps_above_default_boundary() {
        // Four chunks scored 1,2,3,5 at the default min_score (2): 1 and 2 removed, 3 and 5
        // kept (3 > 2 exercises the > path at the real default). Verifies multi-removal and
        // that every removed chunk's incident edges are cleaned up in one pass.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"score": 1}"#,
            r#"{"score": 2}"#,
            r#"{"score": 3}"#,
            r#"{"score": 5}"#,
        ]));
        let graph = doc_with_chunks("S", &[("c1", "a"), ("c2", "b"), ("c3", "c"), ("c4", "d")]);
        let filtered = CustomNodeFilter::new(llm)
            .filter(graph)
            .await
            .expect("filter");

        assert!(filtered.node("c1").is_none() && filtered.node("c2").is_none());
        assert!(filtered.node("c3").is_some() && filtered.node("c4").is_some());
        // Only the two kept chunks' `contains` edges survive; none reference a removed chunk.
        assert_eq!(filtered.edges_by_relationship("contains").len(), 2);
        assert!(filtered.edges.iter().all(|edge| {
            !["c1", "c2"].contains(&edge.source_id.as_str())
                && !["c1", "c2"].contains(&edge.target_id.as_str())
        }));
    }

    #[tokio::test]
    async fn custom_node_filter_never_scores_non_chunk_nodes() {
        // A non-"chunk" node carrying its own summary+text is never scored or removed — the
        // filter is scoped to chunks only.
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let graph = KnowledgeGraph::new().add_node(
            GraphNode::new("section", "section")
                .with_property("summary", GraphProperty::Text("a summary".to_string()))
                .with_property("text", GraphProperty::Text("some content".to_string())),
        );
        let filtered = CustomNodeFilter::new(llm.clone())
            .filter(graph)
            .await
            .expect("filter");
        assert!(
            filtered.node("section").is_some(),
            "non-chunk node must be kept"
        );
        assert!(
            llm.prompts().is_empty(),
            "non-chunk node must not be scored"
        );
    }

    #[tokio::test]
    async fn custom_node_filter_prompt_includes_summary_content_and_rubric() {
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"score": 5}"#]));
        let graph = doc_with_chunks(
            "DOC_SUMMARY_MARKER about RAG.",
            &[("c1", "CHUNK_CONTENT_MARKER text")],
        );
        CustomNodeFilter::new(llm.clone())
            .filter(graph)
            .await
            .expect("filter");
        let prompt = &llm.prompts()[0];
        assert!(
            prompt.contains("DOC_SUMMARY_MARKER"),
            "prompt must carry the parent summary"
        );
        assert!(
            prompt.contains("CHUNK_CONTENT_MARKER"),
            "prompt must carry the chunk content"
        );
        // The rubric is present verbatim (first + last of the five default descriptions).
        assert!(prompt.contains(DEFAULT_NODE_FILTER_RUBRICS[0]));
        assert!(prompt.contains(DEFAULT_NODE_FILTER_RUBRICS[4]));
    }

    #[tokio::test]
    async fn custom_node_filter_parses_float_score() {
        // A float score (e.g. 2.0) is rounded to an int; 2 <= min_score 2 -> removed.
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"score": 2.0}"#]));
        let graph = doc_with_chunks("S", &[("c1", "text")]);
        let filtered = CustomNodeFilter::new(llm)
            .filter(graph)
            .await
            .expect("filter");
        assert!(
            filtered.node("c1").is_none(),
            "score 2.0 -> 2 <= 2 -> removed"
        );
    }

    #[tokio::test]
    async fn custom_node_filter_respects_min_score_boundary() {
        // With min_score 3: a chunk scored exactly 3 is removed (<=), a 4 is kept.
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"score": 3}"#, r#"{"score": 4}"#]));
        let graph = doc_with_chunks("S", &[("at", "a"), ("above", "b")]);
        let filtered = CustomNodeFilter::new(llm)
            .with_min_score(3)
            .filter(graph)
            .await
            .expect("filter");
        assert!(
            filtered.node("at").is_none(),
            "score 3 <= min_score 3 -> removed"
        );
        assert!(filtered.node("above").is_some(), "score 4 > 3 -> kept");
    }

    #[tokio::test]
    async fn custom_node_filter_malformed_score_errors() {
        let llm = Arc::new(ScriptedLlm::new(vec!["not json"]));
        let graph = doc_with_chunks("S", &[("c1", "text")]);
        let result = CustomNodeFilter::new(llm).filter(graph).await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    /// Live gate (env-gated): the real model scores an on-topic chunk high (kept) and an
    /// irrelevant chunk low (removed) against the document summary.
    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; run with --ignored"]
    async fn live_custom_node_filter_drops_irrelevant_chunk() {
        let Some(client) = crate::ProviderConfig::from_env().chat_client() else {
            eprintln!("skipping live node filter: OPENAI_API_KEY not set");
            return;
        };
        let llm: Arc<dyn LlmProvider> = Arc::new(client);
        let graph = doc_with_chunks(
            "A technical guide to evaluating retrieval-augmented generation (RAG) systems with \
metrics such as faithfulness, context precision, and answer relevancy.",
            &[
                (
                    "relevant",
                    "Faithfulness measures whether the generated answer is grounded in the \
retrieved context, penalizing unsupported claims.",
                ),
                (
                    "junk",
                    "Buy cheap discount sunglasses now! Limited time offer, click here to win a \
free vacation!!!",
                ),
            ],
        );
        let filtered = CustomNodeFilter::new(llm)
            .filter(graph)
            .await
            .expect("live filter");
        assert!(
            filtered.node("relevant").is_some(),
            "the on-topic chunk should be kept"
        );
        assert!(
            filtered.node("junk").is_none(),
            "the irrelevant chunk should be scored low and removed"
        );
    }

    #[tokio::test]
    async fn apply_transforms_threads_extract_then_build() {
        // [Extract(NER on chunks) -> Overlap]: the engine writes entity properties, then the
        // builder uses them. "zzz" is the noisy item, so the shared "Tesla" drives an edge.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"entities": ["zzz", "Tesla", "Foo"]}"#,
            r#"{"entities": ["zzz", "Tesla", "Bar"]}"#,
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(GraphNode::new("doc", "document"))
            .add_node(text_node("c1", "about Tesla"))
            .add_node(text_node("c2", "also Tesla"))
            .add_edge(GraphEdge::new("doc", "c1", "contains"))
            .add_edge(GraphEdge::new("doc", "c2", "contains"));

        let out = apply_transforms(
            graph,
            vec![
                GraphTransform::extract(LlmExtractor::new(llm, LlmExtractorKind::Ner))
                    .for_node_type("chunk"),
                GraphTransform::overlap(0.9, 0.01),
            ],
        )
        .await
        .expect("pipeline");

        // Entities were written on the chunks (not the doc), then an overlap edge was built.
        assert!(matches!(
            out.node("c1").unwrap().properties.get("entities"),
            Some(GraphProperty::TextList(_))
        ));
        assert!(!out.node("doc").unwrap().properties.contains_key("entities"));
        assert_eq!(out.edges_by_relationship("entities_overlap").len(), 1);
    }

    #[tokio::test]
    async fn apply_transforms_parallel_applies_every_child() {
        // A Parallel group runs all its children (sequentially, same result): both NER and
        // Themes properties end up on the chunk.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"entities": ["E1"]}"#,
            r#"{"output": ["T1"]}"#,
        ]));
        let graph = KnowledgeGraph::new().add_node(text_node("c1", "content"));
        let out = apply_transforms(
            graph,
            vec![GraphTransform::parallel(vec![
                GraphTransform::extract(LlmExtractor::new(llm.clone(), LlmExtractorKind::Ner)),
                GraphTransform::extract(LlmExtractor::new(llm, LlmExtractorKind::Themes)),
            ])],
        )
        .await
        .expect("pipeline");

        let props = &out.node("c1").unwrap().properties;
        assert_eq!(
            props.get("entities"),
            Some(&GraphProperty::TextList(vec!["E1".to_string()]))
        );
        assert_eq!(
            props.get("themes"),
            Some(&GraphProperty::TextList(vec!["T1".to_string()]))
        );
    }

    #[tokio::test]
    async fn apply_transforms_runs_filter_step() {
        // A Filter step in the pipeline drops a low-scoring chunk.
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"score": 5}"#, r#"{"score": 1}"#]));
        let graph = doc_with_chunks("RAG eval guide.", &[("keep", "good"), ("drop", "bad")]);
        let out = apply_transforms(
            graph,
            vec![GraphTransform::filter(CustomNodeFilter::new(llm))],
        )
        .await
        .expect("pipeline");
        assert!(out.node("keep").is_some());
        assert!(out.node("drop").is_none());
    }

    #[tokio::test]
    async fn apply_transforms_node_type_filter_restricts_scoring() {
        // Extract restricted to chunks: the doc node (also has text) is never scored.
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"entities": ["X"]}"#]));
        let graph = KnowledgeGraph::new()
            .add_node(
                text_node("doc", "doc text").with_property("extra", GraphProperty::Boolean(true)),
            )
            .add_node(text_node("c1", "chunk text"));
        // Make the doc a non-chunk type.
        let graph = KnowledgeGraph {
            nodes: graph
                .nodes
                .into_iter()
                .map(|mut node| {
                    if node.id == "doc" {
                        node.node_type = "document".to_string();
                    }
                    node
                })
                .collect(),
            edges: graph.edges,
        };
        let out = apply_transforms(
            graph,
            vec![
                GraphTransform::extract(LlmExtractor::new(llm.clone(), LlmExtractorKind::Ner))
                    .for_node_type("chunk"),
            ],
        )
        .await
        .expect("pipeline");

        assert!(out.node("c1").unwrap().properties.contains_key("entities"));
        assert!(!out.node("doc").unwrap().properties.contains_key("entities"));
        // Exactly one scoring call (the single chunk), proving the doc was skipped.
        assert_eq!(llm.prompts().len(), 1);
    }

    #[tokio::test]
    async fn apply_transforms_empty_pipeline_is_identity() {
        let graph = doc_with_chunks("S", &[("c1", "a"), ("c2", "b")]);
        let out = apply_transforms(graph.clone(), vec![])
            .await
            .expect("identity");
        assert_eq!(out, graph, "an empty pipeline returns the graph unchanged");
    }

    #[tokio::test]
    async fn apply_transforms_three_stage_extract_extract_then_build() {
        // [Extract(NER) -> Extract(Themes) -> Overlap]: both property steps run before the
        // builder, and the builder consumes the entities the first step wrote.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"entities": ["zzz", "Tesla", "Foo"]}"#, // c1 NER
            r#"{"entities": ["zzz", "Tesla", "Bar"]}"#, // c2 NER
            r#"{"output": ["theme-a"]}"#,               // c1 Themes
            r#"{"output": ["theme-b"]}"#,               // c2 Themes
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(text_node("c1", "x"))
            .add_node(text_node("c2", "y"));
        let out = apply_transforms(
            graph,
            vec![
                GraphTransform::extract(LlmExtractor::new(llm.clone(), LlmExtractorKind::Ner)),
                GraphTransform::extract(LlmExtractor::new(llm, LlmExtractorKind::Themes)),
                GraphTransform::overlap(0.9, 0.01),
            ],
        )
        .await
        .expect("pipeline");

        // Both extracted properties are present, and the overlap edge used the entities.
        assert!(out.node("c1").unwrap().properties.contains_key("entities"));
        assert!(out.node("c1").unwrap().properties.contains_key("themes"));
        assert_eq!(out.edges_by_relationship("entities_overlap").len(), 1);
    }

    #[tokio::test]
    async fn apply_transforms_nested_parallel_runs_every_leaf() {
        // Parallel(Parallel(NER, Themes), Title) -> all three properties land (exercises the
        // recursive Parallel handling).
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"entities": ["E"]}"#,
            r#"{"output": ["T"]}"#,
            r#"{"text": "A Title"}"#,
        ]));
        let graph = KnowledgeGraph::new().add_node(text_node("c1", "content"));
        let out = apply_transforms(
            graph,
            vec![GraphTransform::parallel(vec![
                GraphTransform::parallel(vec![
                    GraphTransform::extract(LlmExtractor::new(llm.clone(), LlmExtractorKind::Ner)),
                    GraphTransform::extract(LlmExtractor::new(
                        llm.clone(),
                        LlmExtractorKind::Themes,
                    )),
                ]),
                GraphTransform::extract(LlmExtractor::new(llm, LlmExtractorKind::Title)),
            ])],
        )
        .await
        .expect("pipeline");

        let props = &out.node("c1").unwrap().properties;
        assert!(props.contains_key("entities"));
        assert!(props.contains_key("themes"));
        assert_eq!(
            props.get("title"),
            Some(&GraphProperty::Text("A Title".to_string()))
        );
    }

    #[tokio::test]
    async fn apply_transforms_extract_without_node_type_touches_all_nodes() {
        // No node-type filter -> every node (doc included) is extracted.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"entities": ["A"]}"#,
            r#"{"entities": ["B"]}"#,
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(text_node("doc", "doc text"))
            .add_node(text_node("c1", "chunk text"));
        let out = apply_transforms(
            graph,
            vec![GraphTransform::extract(LlmExtractor::new(
                llm.clone(),
                LlmExtractorKind::Ner,
            ))],
        )
        .await
        .expect("pipeline");
        assert!(out.node("doc").unwrap().properties.contains_key("entities"));
        assert!(out.node("c1").unwrap().properties.contains_key("entities"));
        assert_eq!(llm.prompts().len(), 2, "both nodes were extracted");
    }

    #[tokio::test]
    async fn apply_transforms_filter_then_builder_sees_filtered_graph() {
        // [Filter -> Overlap]: the filter drops a chunk, so the later builder only sees the
        // survivors and never builds an edge to the removed node. Three chunks are scored in
        // node order (drop, keep, other): drop=1 removed, keep=5 and other=5 kept.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"score": 1}"#,
            r#"{"score": 5}"#,
            r#"{"score": 5}"#,
        ]));
        let graph = doc_with_chunks("RAG eval.", &[("drop", "bad"), ("keep", "good")])
            .add_node(text_node("other", "good too").with_property(
                "entities",
                GraphProperty::TextList(vec!["zzz".to_string(), "Shared".to_string()]),
            ))
            .add_edge(GraphEdge::new("doc", "other", "contains"));
        // Give the two surviving chunks an overlapping entity so Overlap *could* link them.
        let graph = KnowledgeGraph {
            nodes: graph
                .nodes
                .into_iter()
                .map(|mut node| {
                    if node.id == "keep" {
                        node.properties.insert(
                            "entities".to_string(),
                            GraphProperty::TextList(vec!["zzz".to_string(), "Shared".to_string()]),
                        );
                    }
                    node
                })
                .collect(),
            edges: graph.edges,
        };

        let out = apply_transforms(
            graph,
            vec![
                GraphTransform::filter(CustomNodeFilter::new(llm)),
                GraphTransform::overlap(0.9, 0.01),
            ],
        )
        .await
        .expect("pipeline");

        assert!(
            out.node("drop").is_none(),
            "the low-scoring chunk was filtered out"
        );
        // No overlap edge references the removed node; the surviving pair (keep, other) links.
        let overlaps = out.edges_by_relationship("entities_overlap");
        assert!(
            overlaps
                .iter()
                .all(|edge| edge.source_id != "drop" && edge.target_id != "drop")
        );
        assert_eq!(
            overlaps.len(),
            1,
            "only the two survivors sharing 'Shared' link"
        );
    }

    /// Live gate (env-gated): a real pipeline `[Embed(chunks) -> Cosine]` through the engine
    /// gives the chunks embeddings and links the two similar ones, while the embed-less doc is
    /// skipped by the node-type filter.
    #[tokio::test]
    #[ignore = "requires embedding provider env; run with --ignored"]
    async fn live_apply_transforms_embed_then_cosine_pipeline() {
        let Some(client) = crate::ProviderConfig::from_env().embedding_client() else {
            eprintln!("skipping live transforms engine: embedding provider not set");
            return;
        };
        let embedding: Arc<dyn EmbeddingProvider> = Arc::new(client);
        let graph = KnowledgeGraph::new()
            .add_node(GraphNode::new("doc", "document"))
            .add_node(text_node(
                "c1",
                "Cats are small domestic felines kept as pets.",
            ))
            .add_node(text_node("c2", "Domestic cats are popular household pets."))
            .add_edge(GraphEdge::new("doc", "c1", "contains"))
            .add_edge(GraphEdge::new("doc", "c2", "contains"));

        let out = apply_transforms(
            graph,
            vec![
                GraphTransform::embed(EmbeddingExtractor::new(embedding)).for_node_type("chunk"),
                GraphTransform::cosine(0.5),
            ],
        )
        .await
        .expect("live pipeline");

        // Chunks embedded, doc skipped by the node-type filter, similar chunks linked.
        assert!(matches!(
            out.node("c1").unwrap().properties.get("embedding"),
            Some(GraphProperty::Vector(_))
        ));
        assert!(
            !out.node("doc")
                .unwrap()
                .properties
                .contains_key("embedding")
        );
        assert_eq!(out.edges_by_relationship("cosine_similarity").len(), 1);
    }

    /// A node carrying a summary + summary_embedding, the layout `generate_personas_from_kg`
    /// clusters over.
    fn summarized_node(id: &str, summary: &str, embedding: Vec<f32>) -> GraphNode {
        GraphNode::new(id, "chunk")
            .with_property("summary", GraphProperty::Text(summary.to_string()))
            .with_property("summary_embedding", GraphProperty::Vector(embedding))
    }

    #[tokio::test]
    async fn generate_personas_from_kg_clusters_and_uses_longest_representative() {
        // n0 and n1 share an embedding (cosine 1.0 > 0.75) -> one cluster whose representative is
        // the LONGER summary (n1); n2 is orthogonal -> its own cluster. Two personas, in order.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"name": "Pet Owner", "role_description": "Cares for domestic cats."}"#,
            r#"{"name": "Investor", "role_description": "Follows the markets."}"#,
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(summarized_node("n0", "Cats.", vec![1.0, 0.0]))
            .add_node(summarized_node(
                "n1",
                "Cats are domestic felines kept as pets.",
                vec![1.0, 0.0],
            ))
            .add_node(summarized_node(
                "n2",
                "The stock market fell.",
                vec![0.0, 1.0],
            ));

        let personas = generate_personas_from_kg(llm.clone(), &graph, 3)
            .await
            .expect("personas");
        assert_eq!(
            personas.len(),
            2,
            "two clusters -> two personas (no random padding)"
        );
        assert_eq!(personas[0].name, "Pet Owner");
        assert_eq!(personas[0].role, "Cares for domestic cats.");
        assert!(
            personas[0].goals.is_empty(),
            "KG personas leave goals empty"
        );
        // The first cluster's prompt carried the LONGER summary (n1), not n0's "Cats.".
        assert!(llm.prompts()[0].contains("Cats are domestic felines kept as pets."));
        // The second cluster (n2) used its own summary, and produced the second persona.
        assert!(llm.prompts()[1].contains("The stock market fell."));
        assert_eq!(personas[1].name, "Investor");
    }

    #[tokio::test]
    async fn generate_personas_from_kg_clusters_are_anchor_based_not_transitive() {
        // Angles 0deg / 40deg / 80deg: cos40 ~= 0.766 > 0.75 so a~b and b~c, but cos80 ~= 0.174
        // so a !~ c. Anchor-based clustering groups c separately (it is only compared to anchor a),
        // -> 2 clusters. A *transitive* (union-find) clustering would merge all three into one.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"name": "A", "role_description": "a"}"#,
            r#"{"name": "C", "role_description": "c"}"#,
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(summarized_node("a", "anchor a", vec![1.0, 0.0]))
            .add_node(summarized_node("b", "mid b", vec![0.766, 0.643]))
            .add_node(summarized_node("c", "far c", vec![0.174, 0.985]));
        let personas = generate_personas_from_kg(llm.clone(), &graph, 3)
            .await
            .expect("personas");
        assert_eq!(
            personas.len(),
            2,
            "non-transitive: {{a,b}} and {{c}}, not one merged cluster"
        );
        // First persona from cluster {a,b} (rep = longer of "anchor a"/"mid b" = "anchor a"),
        // second from {c}.
        assert!(llm.prompts()[1].contains("far c"));
    }

    #[tokio::test]
    async fn generate_personas_from_kg_num_personas_zero_returns_empty() {
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let graph = KnowledgeGraph::new().add_node(summarized_node("n0", "s", vec![1.0, 0.0]));
        let personas = generate_personas_from_kg(llm.clone(), &graph, 0)
            .await
            .expect("personas");
        assert!(personas.is_empty());
        assert!(
            llm.prompts().is_empty(),
            "num_personas=0 makes no LLM calls"
        );
    }

    #[tokio::test]
    async fn generate_personas_from_kg_ignores_a_goals_field_in_output() {
        // Even if the model volunteers a `goals` array, KG personas keep goals empty.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"name": "N", "role_description": "R", "goals": ["x", "y"]}"#,
        ]));
        let graph = KnowledgeGraph::new().add_node(summarized_node("n0", "s", vec![1.0, 0.0]));
        let personas = generate_personas_from_kg(llm, &graph, 1)
            .await
            .expect("personas");
        assert_eq!(personas[0].name, "N");
        assert!(
            personas[0].goals.is_empty(),
            "the volunteered goals field is ignored"
        );
    }

    #[tokio::test]
    async fn generate_personas_from_kg_requires_role_description_key() {
        // The parser expects `role_description`, not `role`; the wrong key is a parse error.
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"name": "N", "role": "R"}"#]));
        let graph = KnowledgeGraph::new().add_node(summarized_node("n0", "s", vec![1.0, 0.0]));
        let Err(RagasError::Parse { message }) = generate_personas_from_kg(llm, &graph, 1).await
        else {
            panic!("expected a Parse error for the wrong role key");
        };
        assert!(message.contains("role_description"), "message: {message}");
    }

    #[tokio::test]
    async fn generate_personas_from_kg_skips_embedding_without_summary() {
        // Both nodes are ineligible — one has a summary but no embedding, the other an embedding
        // but no summary — so the function errors (no eligible node).
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let graph = KnowledgeGraph::new()
            .add_node(
                GraphNode::new("n0", "chunk")
                    .with_property("summary", GraphProperty::Text("no embedding".to_string())),
            )
            .add_node(
                GraphNode::new("n1", "chunk")
                    .with_property("summary_embedding", GraphProperty::Vector(vec![1.0, 0.0])),
            );
        let result = generate_personas_from_kg(llm, &graph, 3).await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn generate_personas_from_kg_longest_summary_tie_keeps_first() {
        // Two same-embedding nodes with equal-length summaries -> one cluster; the FIRST is the
        // representative (strict `>` keeps the earliest on a length tie).
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"name": "N", "role_description": "R"}"#,
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(summarized_node("first", "AAAA", vec![1.0, 0.0]))
            .add_node(summarized_node("second", "BBBB", vec![1.0, 0.0]));
        generate_personas_from_kg(llm.clone(), &graph, 1)
            .await
            .expect("personas");
        assert!(
            llm.prompts()[0].contains("AAAA") && !llm.prompts()[0].contains("BBBB"),
            "tie -> first summary is the representative, got: {}",
            llm.prompts()[0]
        );
    }

    #[tokio::test]
    async fn generate_personas_from_kg_skips_nodes_without_summary_embedding() {
        // Only n1 has both summary + summary_embedding; n0 (summary, no embedding) is ignored.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"name": "Reader", "role_description": "Reads the guide."}"#,
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(
                GraphNode::new("n0", "chunk")
                    .with_property("summary", GraphProperty::Text("no embedding".to_string())),
            )
            .add_node(summarized_node("n1", "has both", vec![1.0, 0.0]));
        let personas = generate_personas_from_kg(llm.clone(), &graph, 3)
            .await
            .expect("personas");
        assert_eq!(personas.len(), 1);
        assert_eq!(llm.prompts().len(), 1, "only the eligible node was used");
    }

    #[tokio::test]
    async fn generate_personas_from_kg_errors_when_no_eligible_nodes() {
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let graph = KnowledgeGraph::new().add_node(text_node("c1", "text but no summary"));
        let result = generate_personas_from_kg(llm, &graph, 3).await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn generate_personas_from_kg_caps_at_num_personas() {
        // Three orthogonal clusters but num_personas = 2 -> two personas, two LLM calls.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"name": "A", "role_description": "a"}"#,
            r#"{"name": "B", "role_description": "b"}"#,
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(summarized_node("n0", "alpha", vec![1.0, 0.0, 0.0]))
            .add_node(summarized_node("n1", "beta", vec![0.0, 1.0, 0.0]))
            .add_node(summarized_node("n2", "gamma", vec![0.0, 0.0, 1.0]));
        let personas = generate_personas_from_kg(llm.clone(), &graph, 2)
            .await
            .expect("personas");
        assert_eq!(personas.len(), 2);
        assert_eq!(llm.prompts().len(), 2);
    }

    #[tokio::test]
    async fn generate_personas_from_kg_malformed_output_errors() {
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"name": ""}"#]));
        let graph = KnowledgeGraph::new().add_node(summarized_node("n0", "s", vec![1.0]));
        // Empty name (and missing role_description) -> a typed parse error.
        let result = generate_personas_from_kg(llm, &graph, 1).await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    /// Live gate (env-gated): the real model turns a KG node summary into a usable persona
    /// (non-empty name + role description).
    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; run with --ignored"]
    async fn live_generate_personas_from_kg_builds_a_persona() {
        let Some(client) = crate::ProviderConfig::from_env().chat_client() else {
            eprintln!("skipping live persona generation: OPENAI_API_KEY not set");
            return;
        };
        let llm: Arc<dyn LlmProvider> = Arc::new(client);
        // Two near-identical embeddings -> one cluster -> one persona from the longer summary.
        let graph = KnowledgeGraph::new()
            .add_node(summarized_node(
                "n0",
                "A guide to digital marketing strategies for engaging online audiences across \
social platforms, SEO, and email campaigns.",
                vec![1.0, 0.0],
            ))
            .add_node(summarized_node("n1", "Marketing overview.", vec![1.0, 0.0]));

        let personas = generate_personas_from_kg(llm, &graph, 1)
            .await
            .expect("live personas");
        assert_eq!(personas.len(), 1);
        assert!(!personas[0].name.trim().is_empty(), "persona needs a name");
        assert!(
            !personas[0].role.trim().is_empty(),
            "persona needs a role description"
        );
    }

    fn persona(name: &str, role: &str) -> Persona {
        Persona {
            name: name.to_string(),
            role: role.to_string(),
            goals: Vec::new(),
        }
    }

    fn entitied_chunk(id: &str, entities: &[&str]) -> GraphNode {
        GraphNode::new(id, "chunk").with_property(
            "entities",
            GraphProperty::TextList(entities.iter().map(|e| e.to_string()).collect()),
        )
    }

    #[tokio::test]
    async fn match_themes_to_personas_parses_mapping() {
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Chef": ["cooking"], "Coder": ["rust", "apis"]}}"#,
        ]));
        let llm_dyn: Arc<dyn LlmProvider> = llm.clone();
        let mapping = match_themes_to_personas(
            &llm_dyn,
            &["cooking".to_string(), "rust".to_string()],
            &[persona("Chef", "cooks"), persona("Coder", "writes code")],
        )
        .await
        .expect("mapping");
        assert_eq!(mapping.get("Chef"), Some(&vec!["cooking".to_string()]));
        assert_eq!(
            mapping.get("Coder"),
            Some(&vec!["rust".to_string(), "apis".to_string()])
        );
        // The prompt carried the themes and persona roles.
        assert!(llm.prompts()[0].contains("cooking") && llm.prompts()[0].contains("writes code"));
    }

    #[tokio::test]
    async fn match_themes_to_personas_malformed_errors() {
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![r#"{"no_mapping": 1}"#]));
        let result = match_themes_to_personas(&llm, &["a".to_string()], &[persona("P", "r")]).await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_pairs_terms_with_matching_personas() {
        // One chunk with two entities; mapping sends "galaxy" -> Astronomer, "stew" -> Chef.
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Astronomer": ["galaxy"], "Chef": ["stew"]}}"#,
        ]));
        let graph = KnowledgeGraph::new().add_node(entitied_chunk("c1", &["galaxy", "stew"]));
        let personas = [
            persona("Astronomer", "studies space"),
            persona("Chef", "cooks"),
        ];

        let scenarios = prepare_single_hop_scenarios(&llm, &graph, &personas, 10)
            .await
            .expect("scenarios");
        assert_eq!(scenarios.len(), 2);
        let galaxy = scenarios
            .iter()
            .find(|s| s.term == "galaxy")
            .expect("galaxy");
        assert_eq!(galaxy.persona.name, "Astronomer");
        assert_eq!(galaxy.node_id, "c1");
        let stew = scenarios.iter().find(|s| s.term == "stew").expect("stew");
        assert_eq!(stew.persona.name, "Chef");
        // Style/length rotate across the produced scenarios (first two differ in style).
        assert_eq!(scenarios[0].style, QueryStyle::ALL[0]);
        assert_eq!(scenarios[1].style, QueryStyle::ALL[1]);
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_skips_unmatched_terms() {
        // Only "rust" is matched; "weather" maps to nobody -> no scenario for it.
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Coder": ["rust"]}}"#,
        ]));
        let graph = KnowledgeGraph::new().add_node(entitied_chunk("c1", &["rust", "weather"]));
        let scenarios =
            prepare_single_hop_scenarios(&llm, &graph, &[persona("Coder", "codes")], 10)
                .await
                .expect("scenarios");
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].term, "rust");
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_selects_majority_node_type() {
        // Two chunks-with-entities vs one doc-with-entities -> chunks win; the doc is ignored.
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"P": ["a"]}}"#,
            r#"{"mapping": {"P": ["b"]}}"#,
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(entitied_chunk("c1", &["a"]))
            .add_node(entitied_chunk("c2", &["b"]))
            .add_node(
                GraphNode::new("d1", "document")
                    .with_property("entities", GraphProperty::TextList(vec!["z".to_string()])),
            );
        let scenarios = prepare_single_hop_scenarios(&llm, &graph, &[persona("P", "r")], 10)
            .await
            .expect("scenarios");
        assert!(
            scenarios
                .iter()
                .all(|s| s.node_id == "c1" || s.node_id == "c2")
        );
        assert!(scenarios.iter().all(|s| s.node_id != "d1"));
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_caps_at_n_and_per_node() {
        // n = 1 over a single 2-entity node -> samples_per_node = 1 -> exactly one scenario, and
        // only one theme-matching call.
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"mapping": {"P": ["a", "b"]}}"#]));
        let llm_dyn: Arc<dyn LlmProvider> = llm.clone();
        let graph = KnowledgeGraph::new().add_node(entitied_chunk("c1", &["a", "b"]));
        let scenarios = prepare_single_hop_scenarios(&llm_dyn, &graph, &[persona("P", "r")], 1)
            .await
            .expect("scenarios");
        assert_eq!(scenarios.len(), 1);
        assert_eq!(llm.prompts().len(), 1);
        // The single scenario is well-formed, not just present.
        assert_eq!(scenarios[0].node_id, "c1");
        assert_eq!(scenarios[0].term, "a");
        assert_eq!(scenarios[0].persona.name, "P");
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_distributes_across_nodes_and_rotates() {
        // n=5 over two 3-entity nodes: samples_per_node = ceil(5/2) = 3, so c1 contributes 3 and
        // c2 contributes 2 (global cap), total 5. Style/length rotate (and wrap) across all five.
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"P": ["a", "b", "c"]}}"#,
            r#"{"mapping": {"P": ["d", "e", "f"]}}"#,
        ]));
        let graph = KnowledgeGraph::new()
            .add_node(entitied_chunk("c1", &["a", "b", "c"]))
            .add_node(entitied_chunk("c2", &["d", "e", "f"]));
        let scenarios = prepare_single_hop_scenarios(&llm, &graph, &[persona("P", "r")], 5)
            .await
            .expect("scenarios");
        assert_eq!(scenarios.len(), 5, "total capped at n");
        assert_eq!(
            scenarios.iter().filter(|s| s.node_id == "c1").count(),
            3,
            "first node fills ceil(5/2)=3"
        );
        assert_eq!(
            scenarios.iter().filter(|s| s.node_id == "c2").count(),
            2,
            "second node gets the remaining 2"
        );
        // Style wraps at 4 (4 variants), length cycles every 3.
        assert_eq!(scenarios[0].style, QueryStyle::Misspelled);
        assert_eq!(scenarios[4].style, QueryStyle::Misspelled);
        assert_eq!(scenarios[3].length, QueryLength::Long);
        assert_eq!(scenarios[4].length, QueryLength::Medium);
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_matches_terms_case_insensitively() {
        // Node entity "Galaxy" vs lowercase mapping "galaxy" still matches (eq_ignore_ascii_case).
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Astro": ["galaxy"]}}"#,
        ]));
        let graph = KnowledgeGraph::new().add_node(entitied_chunk("c1", &["Galaxy"]));
        let scenarios = prepare_single_hop_scenarios(&llm, &graph, &[persona("Astro", "r")], 5)
            .await
            .expect("scenarios");
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].term, "Galaxy");
        assert_eq!(scenarios[0].persona.name, "Astro");
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_tie_favors_documents() {
        // Equal chunk/document counts (1 each) -> documents win (per get_node_clusters).
        let llm: Arc<dyn LlmProvider> =
            Arc::new(ScriptedLlm::new(vec![r#"{"mapping": {"P": ["d"]}}"#]));
        let graph = KnowledgeGraph::new()
            .add_node(entitied_chunk("c1", &["x"]))
            .add_node(
                GraphNode::new("doc1", "document")
                    .with_property("entities", GraphProperty::TextList(vec!["d".to_string()])),
            );
        let scenarios = prepare_single_hop_scenarios(&llm, &graph, &[persona("P", "r")], 5)
            .await
            .expect("scenarios");
        assert!(
            scenarios.iter().all(|s| s.node_id == "doc1"),
            "ties favor documents"
        );
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_skips_empty_entities_node() {
        // A chunk with an empty entities list is not eligible; only the non-empty one is used.
        let llm: Arc<dyn LlmProvider> =
            Arc::new(ScriptedLlm::new(vec![r#"{"mapping": {"P": ["a"]}}"#]));
        let graph = KnowledgeGraph::new()
            .add_node(entitied_chunk("c1", &["a"]))
            .add_node(entitied_chunk("empty", &[]));
        let scenarios = prepare_single_hop_scenarios(&llm, &graph, &[persona("P", "r")], 5)
            .await
            .expect("scenarios");
        assert!(scenarios.iter().all(|s| s.node_id == "c1"));
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_picks_first_matching_persona_on_tie() {
        // Both personas map to the same term -> the first in the list is chosen.
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"First": ["shared"], "Second": ["shared"]}}"#,
        ]));
        let graph = KnowledgeGraph::new().add_node(entitied_chunk("c1", &["shared"]));
        let scenarios = prepare_single_hop_scenarios(
            &llm,
            &graph,
            &[persona("First", "r"), persona("Second", "r")],
            5,
        )
        .await
        .expect("scenarios");
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].persona.name, "First");
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_reuses_persona_across_a_nodes_terms() {
        // One persona matched to two of a node's terms -> both scenarios use that persona.
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Astro": ["galaxy", "telescope"]}}"#,
        ]));
        let graph = KnowledgeGraph::new().add_node(entitied_chunk("c1", &["galaxy", "telescope"]));
        let scenarios = prepare_single_hop_scenarios(&llm, &graph, &[persona("Astro", "r")], 5)
            .await
            .expect("scenarios");
        assert_eq!(scenarios.len(), 2);
        assert!(scenarios.iter().all(|s| s.persona.name == "Astro"));
    }

    #[tokio::test]
    async fn match_themes_to_personas_non_array_value_errors() {
        // A persona value that is a bare string (not a list) is rejected, matching Python's
        // pydantic Dict[str, List[str]] validation rather than silently dropping it.
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Chef": "cooking"}}"#,
        ]));
        let result =
            match_themes_to_personas(&llm, &["cooking".to_string()], &[persona("Chef", "r")]).await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn prepare_single_hop_scenarios_errors_without_entity_nodes() {
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![]));
        let graph = KnowledgeGraph::new().add_node(text_node("c1", "no entities"));
        let result = prepare_single_hop_scenarios(&llm, &graph, &[persona("P", "r")], 5).await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    /// Live gate (env-gated): the real model matches a node's astronomy entities to the
    /// astronomer persona (not the chef), producing scenarios anchored on the relevant persona.
    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; run with --ignored"]
    async fn live_prepare_single_hop_scenarios_matches_relevant_persona() {
        let Some(client) = crate::ProviderConfig::from_env().chat_client() else {
            eprintln!("skipping live scenario prep: OPENAI_API_KEY not set");
            return;
        };
        let llm: Arc<dyn LlmProvider> = Arc::new(client);
        let graph = KnowledgeGraph::new().add_node(entitied_chunk("c1", &["galaxy", "telescope"]));
        let personas = [
            persona("Astronomer", "Studies stars, galaxies, and telescopes."),
            persona("Chef", "Cooks food and develops recipes."),
        ];
        let scenarios = prepare_single_hop_scenarios(&llm, &graph, &personas, 5)
            .await
            .expect("live scenarios");
        assert!(
            !scenarios.is_empty(),
            "expected scenarios from the matched persona"
        );
        assert!(
            scenarios.iter().all(|s| s.persona.name == "Astronomer"),
            "astronomy terms should match the astronomer, not the chef: {scenarios:?}"
        );
        // Each scenario is well-formed: capped at n, anchored on the input node, term drawn from
        // its entities.
        assert!(scenarios.len() <= 5);
        assert!(scenarios.iter().all(|s| s.node_id == "c1"));
        assert!(
            scenarios
                .iter()
                .all(|s| ["galaxy", "telescope"].contains(&s.term.as_str())),
            "terms must come from the node's entities: {scenarios:?}"
        );
    }

    fn entitied_text_chunk(id: &str, entities: &[&str], text: &str) -> GraphNode {
        GraphNode::new(id, "chunk")
            .with_property(
                "entities",
                GraphProperty::TextList(entities.iter().map(|e| e.to_string()).collect()),
            )
            .with_property("text", GraphProperty::Text(text.to_string()))
    }

    #[tokio::test]
    async fn single_hop_specific_synthesizer_builds_grounded_dataset() {
        // One chunk, two entities -> galaxy maps to the Astronomer, stew to the Chef. The
        // mapping call comes first, then one query/answer call per scenario in term order.
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Astronomer": ["galaxy"], "Chef": ["stew"]}}"#,
            r#"{"query": "What is a galaxy?", "answer": "A galaxy is a system of stars."}"#,
            r#"{"query": "How do you make stew?", "answer": "Simmer the ingredients together."}"#,
        ]));
        let graph = KnowledgeGraph::new().add_node(entitied_text_chunk(
            "c1",
            &["galaxy", "stew"],
            "A galaxy is a system of stars. Stew is made by simmering ingredients.",
        ));
        let personas = [
            persona("Astronomer", "studies space"),
            persona("Chef", "cooks"),
        ];

        let dataset = SingleHopSpecificSynthesizer::new(llm)
            .generate(&graph, &personas, 10)
            .await
            .expect("dataset");

        assert_eq!(dataset.len(), 2);
        let galaxy = dataset
            .iter()
            .find(|s| s.metadata.get("term").map(String::as_str) == Some("galaxy"))
            .expect("galaxy sample");
        assert_eq!(galaxy.user_input, "What is a galaxy?");
        assert_eq!(
            galaxy.reference.as_deref(),
            Some("A galaxy is a system of stars.")
        );
        // Python fills reference_contexts; our EvaluationDataset also requires response +
        // retrieved_contexts, so the answer/context are mirrored into them.
        assert_eq!(galaxy.response, "A galaxy is a system of stars.");
        assert_eq!(galaxy.reference_contexts.len(), 1);
        assert_eq!(galaxy.retrieved_contexts, galaxy.reference_contexts);
        assert!(galaxy.reference_contexts[0].contains("galaxy is a system of stars"));
        assert_eq!(
            galaxy.metadata.get("persona_name").map(String::as_str),
            Some("Astronomer")
        );
        assert_eq!(
            galaxy.metadata.get("synthesis_type").map(String::as_str),
            Some("single-hop")
        );
        assert_eq!(
            galaxy.metadata.get("source_node_ids").map(String::as_str),
            Some("c1")
        );

        let stew = dataset
            .iter()
            .find(|s| s.metadata.get("term").map(String::as_str) == Some("stew"))
            .expect("stew sample");
        assert_eq!(
            stew.metadata.get("persona_name").map(String::as_str),
            Some("Chef")
        );

        // Style/length are recorded as the Python variant NAMEs and rotate across scenarios.
        assert_eq!(
            galaxy.metadata.get("query_style").map(String::as_str),
            Some("MISSPELLED")
        );
        assert_eq!(
            galaxy.metadata.get("query_length").map(String::as_str),
            Some("LONG")
        );
        assert_eq!(
            stew.metadata.get("query_style").map(String::as_str),
            Some("PERFECT_GRAMMAR")
        );
        assert_eq!(
            stew.metadata.get("query_length").map(String::as_str),
            Some("MEDIUM")
        );
    }

    #[tokio::test]
    async fn single_hop_specific_synthesizer_skips_node_without_text() {
        // Two entity chunks (so neither is skipped by node selection), but only c1 has text.
        // n=2 -> samples_per_node=1; the c2 scenario is dropped for lacking text rather than
        // failing the whole dataset. Mapping is called per node, then one q/a call for c1.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"P": ["alpha"]}}"#,
            r#"{"mapping": {"P": ["beta"]}}"#,
            r#"{"query": "Q?", "answer": "A."}"#,
        ]));
        let llm_dyn: Arc<dyn LlmProvider> = llm.clone();
        let graph = KnowledgeGraph::new()
            .add_node(entitied_text_chunk("c1", &["alpha"], "alpha content"))
            .add_node(entitied_chunk("c2", &["beta"]));

        let dataset = SingleHopSpecificSynthesizer::new(llm_dyn)
            .generate(&graph, &[persona("P", "r")], 2)
            .await
            .expect("dataset");

        assert_eq!(dataset.len(), 1);
        assert_eq!(
            dataset.samples()[0]
                .metadata
                .get("source_node_ids")
                .map(String::as_str),
            Some("c1")
        );
        // Both nodes' theme-persona mappings are requested (c1 + c2), then exactly one query/answer
        // call for the surviving c1 scenario — pins that c2 is still mapped, only its sample dropped.
        assert_eq!(llm.prompts().len(), 3);
    }

    #[tokio::test]
    async fn single_hop_specific_synthesizer_empty_when_no_persona_matches() {
        // The mapping matches no term -> no scenarios -> no samples -> EmptyDataset (not a panic).
        let llm: Arc<dyn LlmProvider> =
            Arc::new(ScriptedLlm::new(vec![r#"{"mapping": {"P": []}}"#]));
        let graph =
            KnowledgeGraph::new().add_node(entitied_text_chunk("c1", &["alpha"], "alpha content"));

        let result = SingleHopSpecificSynthesizer::new(llm)
            .generate(&graph, &[persona("P", "r")], 5)
            .await;
        assert!(matches!(result, Err(RagasError::EmptyDataset)));
    }

    #[tokio::test]
    async fn single_hop_specific_synthesizer_errors_on_missing_field() {
        // A query/answer response missing the answer field is a parse error, not a silent blank.
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"P": ["alpha"]}}"#,
            r#"{"query": "Q only"}"#,
        ]));
        let graph =
            KnowledgeGraph::new().add_node(entitied_text_chunk("c1", &["alpha"], "alpha content"));

        let result = SingleHopSpecificSynthesizer::new(llm)
            .generate(&graph, &[persona("P", "r")], 5)
            .await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn single_hop_specific_synthesizer_passes_conditions_into_prompt() {
        // The generation prompt carries the persona, term, context, and any llm_context guidance.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Astronomer": ["galaxy"]}}"#,
            r#"{"query": "What is a galaxy?", "answer": "A system of stars."}"#,
        ]));
        let llm_dyn: Arc<dyn LlmProvider> = llm.clone();
        let graph = KnowledgeGraph::new().add_node(entitied_text_chunk(
            "c1",
            &["galaxy"],
            "A galaxy is a system of stars.",
        ));

        SingleHopSpecificSynthesizer::new(llm_dyn)
            .with_llm_context("ask comparison questions")
            .generate(&graph, &[persona("Astronomer", "studies space")], 1)
            .await
            .expect("dataset");

        // prompts[0] is the theme-persona mapping; prompts[1] is the query/answer generation.
        let qa_prompt = &llm.prompts()[1];
        assert!(qa_prompt.contains("galaxy"), "missing term: {qa_prompt}");
        assert!(
            qa_prompt.contains("Astronomer"),
            "missing persona: {qa_prompt}"
        );
        assert!(
            qa_prompt.contains("A galaxy is a system of stars."),
            "missing context: {qa_prompt}"
        );
        assert!(
            qa_prompt.contains("ask comparison questions"),
            "missing llm_context guidance: {qa_prompt}"
        );
    }

    /// Live gate (env-gated): the real model turns an astronomy chunk + an astronomer/chef persona
    /// pair into a grounded single-hop testset — every sample is anchored on the astronomer and
    /// carries a non-empty grounded answer + reference context.
    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; run with --ignored"]
    async fn live_single_hop_specific_synthesizer_generates_grounded_testset() {
        let Some(client) = crate::ProviderConfig::from_env().chat_client() else {
            eprintln!("skipping live single-hop synthesizer: OPENAI_API_KEY not set");
            return;
        };
        let llm: Arc<dyn LlmProvider> = Arc::new(client);
        let passage = "A galaxy is a gravitationally bound system of stars, gas, and dust. \
Astronomers use telescopes to observe distant galaxies and measure their redshift.";
        let graph = KnowledgeGraph::new().add_node(entitied_text_chunk(
            "c1",
            &["galaxy", "telescope"],
            passage,
        ));
        let personas = [
            persona("Astronomer", "Studies stars, galaxies, and telescopes."),
            persona("Chef", "Cooks food and develops recipes."),
        ];

        let dataset = SingleHopSpecificSynthesizer::new(llm)
            .generate(&graph, &personas, 2)
            .await
            .expect("live dataset");

        assert!(!dataset.is_empty());
        for sample in dataset.iter() {
            assert!(!sample.user_input.trim().is_empty(), "empty query");
            assert!(
                sample
                    .reference
                    .as_deref()
                    .is_some_and(|r| !r.trim().is_empty()),
                "empty grounded answer"
            );
            assert!(
                !sample.reference_contexts.is_empty(),
                "no reference context"
            );
            assert_eq!(sample.reference_contexts[0], passage);
            assert_eq!(
                sample.metadata.get("persona_name").map(String::as_str),
                Some("Astronomer"),
                "astronomy terms should anchor on the astronomer, not the chef"
            );
            // Full synthesizer contract: style/length recorded and the term drawn from the node.
            assert!(
                sample.metadata.contains_key("query_style")
                    && sample.metadata.contains_key("query_length"),
                "missing style/length metadata: {:?}",
                sample.metadata
            );
            assert!(
                ["galaxy", "telescope"].contains(
                    &sample
                        .metadata
                        .get("term")
                        .map(String::as_str)
                        .unwrap_or("")
                ),
                "term must come from the node entities: {:?}",
                sample.metadata.get("term")
            );
        }
    }

    /// Build a two-node entity-overlap cluster: two entity+text chunks joined by an
    /// `entities_overlap` edge carrying the given `"x => y"` overlap items.
    fn overlap_cluster_graph(
        a: (&str, &[&str], &str),
        b: (&str, &[&str], &str),
        overlapped: &[&str],
    ) -> KnowledgeGraph {
        KnowledgeGraph::new()
            .add_node(entitied_text_chunk(a.0, a.1, a.2))
            .add_node(entitied_text_chunk(b.0, b.1, b.2))
            .add_edge(GraphEdge::new(a.0, b.0, "entities_overlap").with_property(
                "overlapped_items",
                GraphProperty::TextList(overlapped.iter().map(|s| s.to_string()).collect()),
            ))
    }

    #[test]
    fn extract_overlap_themes_splits_pairs_and_dedupes_first_seen() {
        let items = vec![
            "Einstein => Einstein".to_string(),
            "Bohr => Bohr".to_string(),
            "Einstein => Einstein".to_string(),
            "Microsoft => Microsft".to_string(),
        ];
        assert_eq!(
            extract_overlap_themes(&items),
            vec![
                "Einstein".to_string(),
                "Bohr".to_string(),
                "Microsoft".to_string(),
                "Microsft".to_string()
            ]
        );
    }

    #[test]
    fn entity_overlap_clusters_normalize_smaller_id_first_and_dedup() {
        // Reverse-direction edge (c2 -> c1) is normalized to (c1, c2); a duplicate pair is deduped.
        let graph = KnowledgeGraph::new()
            .add_node(entitied_chunk("c1", &["x"]))
            .add_node(entitied_chunk("c2", &["x"]))
            .add_edge(
                GraphEdge::new("c2", "c1", "entities_overlap").with_property(
                    "overlapped_items",
                    GraphProperty::TextList(vec!["x => x".to_string()]),
                ),
            )
            .add_edge(
                GraphEdge::new("c1", "c2", "entities_overlap").with_property(
                    "overlapped_items",
                    GraphProperty::TextList(vec!["x => x".to_string()]),
                ),
            );
        let clusters = entity_overlap_clusters(&graph);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].node_a, "c1");
        assert_eq!(clusters[0].node_b, "c2");
    }

    #[tokio::test]
    async fn multi_hop_specific_synthesizer_builds_grounded_dataset() {
        // Two chunks sharing the entity "Einstein" -> the theme matches the Historian persona, and
        // both nodes carry it, so the scenario spans both hops.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Historian": ["Einstein"]}}"#,
            r#"{"query": "How did Einstein's relativity get confirmed?", "answer": "Einstein developed relativity, and the 1919 eclipse confirmed it."}"#,
        ]));
        let llm_dyn: Arc<dyn LlmProvider> = llm.clone();
        let graph = overlap_cluster_graph(
            (
                "c1",
                &["Einstein", "relativity"],
                "Einstein developed the theory of relativity.",
            ),
            (
                "c2",
                &["Einstein", "eclipse"],
                "The 1919 solar eclipse confirmed Einstein's theory.",
            ),
            &["Einstein => Einstein"],
        );

        let dataset = MultiHopSpecificSynthesizer::new(llm_dyn)
            .generate(
                &graph,
                &[persona("Historian", "studies science milestones")],
                5,
            )
            .await
            .expect("dataset");

        assert_eq!(dataset.len(), 1);
        // Exactly one theme-persona mapping call + one query/answer generation call.
        assert_eq!(llm.prompts().len(), 2);
        let sample = &dataset.samples()[0];
        assert_eq!(
            sample.user_input,
            "How did Einstein's relativity get confirmed?"
        );
        assert_eq!(
            sample.reference.as_deref(),
            Some("Einstein developed relativity, and the 1919 eclipse confirmed it.")
        );
        // Two hop-tagged reference contexts, mirrored into retrieved_contexts.
        assert_eq!(sample.reference_contexts.len(), 2);
        assert!(sample.reference_contexts[0].starts_with("<1-hop>"));
        assert!(sample.reference_contexts[1].starts_with("<2-hop>"));
        assert!(sample.reference_contexts[0].contains("developed the theory of relativity"));
        assert_eq!(sample.retrieved_contexts, sample.reference_contexts);
        assert_eq!(sample.response, sample.reference.clone().unwrap());
        assert_eq!(
            sample.metadata.get("synthesis_type").map(String::as_str),
            Some("multi-hop")
        );
        assert_eq!(
            sample.metadata.get("source_node_ids").map(String::as_str),
            Some("c1,c2")
        );
        assert_eq!(
            sample.metadata.get("themes").map(String::as_str),
            Some("Einstein")
        );
        assert_eq!(
            sample.metadata.get("persona_name").map(String::as_str),
            Some("Historian")
        );
    }

    #[tokio::test]
    async fn prepare_multi_hop_specific_scenarios_errors_without_overlap_edge() {
        let llm: Arc<dyn LlmProvider> = Arc::new(ScriptedLlm::new(vec![]));
        let graph = KnowledgeGraph::new()
            .add_node(entitied_text_chunk("c1", &["x"], "text one"))
            .add_node(entitied_text_chunk("c2", &["x"], "text two"));
        let result =
            prepare_multi_hop_specific_scenarios(&llm, &graph, &[persona("P", "r")], 5).await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
    }

    #[tokio::test]
    async fn prepare_multi_hop_specific_scenarios_skips_theme_no_cluster_node_carries() {
        // The overlap items name themes that neither node actually lists in `entities` -> no
        // scenario can be anchored, so the result is empty (Python's valid_nodes filter).
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"mapping": {"P": ["X", "Y"]}}"#]));
        let llm_dyn: Arc<dyn LlmProvider> = llm.clone();
        let graph = overlap_cluster_graph(
            ("c1", &["A"], "text one"),
            ("c2", &["B"], "text two"),
            &["X => Y"],
        );
        let scenarios =
            prepare_multi_hop_specific_scenarios(&llm_dyn, &graph, &[persona("P", "r")], 5)
                .await
                .expect("scenarios");
        assert!(scenarios.is_empty());
        // The theme-persona mapping WAS performed (the cluster's themes were extracted); the
        // scenarios are empty only because no node carries them, not because matching was skipped.
        assert_eq!(llm.prompts().len(), 1);
    }

    #[tokio::test]
    async fn prepare_multi_hop_specific_scenarios_distributes_across_clusters_and_rotates() {
        // Two clusters, n=2 -> ceil(2/2)=1 per cluster -> one scenario each; style rotates.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"P": ["alpha"]}}"#,
            r#"{"mapping": {"P": ["beta"]}}"#,
        ]));
        let llm_dyn: Arc<dyn LlmProvider> = llm.clone();
        let graph = KnowledgeGraph::new()
            .add_node(entitied_chunk("c1", &["alpha"]))
            .add_node(entitied_chunk("c2", &["alpha"]))
            .add_node(entitied_chunk("c3", &["beta"]))
            .add_node(entitied_chunk("c4", &["beta"]))
            .add_edge(
                GraphEdge::new("c1", "c2", "entities_overlap").with_property(
                    "overlapped_items",
                    GraphProperty::TextList(vec!["alpha => alpha".to_string()]),
                ),
            )
            .add_edge(
                GraphEdge::new("c3", "c4", "entities_overlap").with_property(
                    "overlapped_items",
                    GraphProperty::TextList(vec!["beta => beta".to_string()]),
                ),
            );
        let scenarios =
            prepare_multi_hop_specific_scenarios(&llm_dyn, &graph, &[persona("P", "r")], 2)
                .await
                .expect("scenarios");
        assert_eq!(scenarios.len(), 2);
        // Each scenario spans both nodes of its cluster.
        assert_eq!(
            scenarios[0].node_ids,
            vec!["c1".to_string(), "c2".to_string()]
        );
        assert_eq!(
            scenarios[1].node_ids,
            vec!["c3".to_string(), "c4".to_string()]
        );
        // Style rotates across the produced scenarios.
        assert_ne!(scenarios[0].style, scenarios[1].style);
        // Two mapping calls (one per cluster), no generation in prep.
        assert_eq!(llm.prompts().len(), 2);
    }

    #[tokio::test]
    async fn multi_hop_specific_synthesizer_passes_hops_and_themes_into_prompt() {
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"Historian": ["Einstein"]}}"#,
            r#"{"query": "q", "answer": "a"}"#,
        ]));
        let llm_dyn: Arc<dyn LlmProvider> = llm.clone();
        let graph = overlap_cluster_graph(
            ("c1", &["Einstein"], "Einstein developed relativity."),
            ("c2", &["Einstein"], "The eclipse confirmed it."),
            &["Einstein => Einstein"],
        );

        MultiHopSpecificSynthesizer::new(llm_dyn)
            .with_llm_context("ask cause-effect questions")
            .generate(&graph, &[persona("Historian", "studies science")], 1)
            .await
            .expect("dataset");

        // Exactly one mapping call + one generation call; prompts[1] is the generation prompt.
        assert_eq!(llm.prompts().len(), 2);
        let qa_prompt = &llm.prompts()[1];
        assert!(
            qa_prompt.contains("<1-hop>"),
            "missing hop tag: {qa_prompt}"
        );
        assert!(
            qa_prompt.contains("<2-hop>"),
            "missing hop tag: {qa_prompt}"
        );
        assert!(qa_prompt.contains("Einstein"), "missing theme: {qa_prompt}");
        assert!(
            qa_prompt.contains("Historian"),
            "missing persona: {qa_prompt}"
        );
        assert!(
            qa_prompt.contains("ask cause-effect questions"),
            "missing llm_context: {qa_prompt}"
        );
    }

    #[tokio::test]
    async fn multi_hop_specific_synthesizer_errors_on_missing_field() {
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"mapping": {"P": ["alpha"]}}"#,
            r#"{"query": "q only"}"#,
        ]));
        let llm_dyn: Arc<dyn LlmProvider> = llm.clone();
        let graph = overlap_cluster_graph(
            ("c1", &["alpha"], "text one"),
            ("c2", &["alpha"], "text two"),
            &["alpha => alpha"],
        );
        let result = MultiHopSpecificSynthesizer::new(llm_dyn)
            .generate(&graph, &[persona("P", "r")], 5)
            .await;
        assert!(matches!(result, Err(RagasError::Parse { .. })));
        // The error came from parsing the generation response, after both the mapping call and the
        // (malformed) generation call were made — not from skipping the LLM entirely.
        assert_eq!(llm.prompts().len(), 2);
    }

    /// Live gate (env-gated): the real model turns an entity-overlap cluster (two chunks sharing
    /// "Einstein") + a historian/chef persona pair into a grounded multi-hop testset — every sample
    /// spans both hop-tagged contexts, is anchored on the historian, and has a grounded answer.
    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; run with --ignored"]
    async fn live_multi_hop_specific_synthesizer_generates_grounded_testset() {
        let Some(client) = crate::ProviderConfig::from_env().chat_client() else {
            eprintln!("skipping live multi-hop synthesizer: OPENAI_API_KEY not set");
            return;
        };
        let llm: Arc<dyn LlmProvider> = Arc::new(client);
        let graph = overlap_cluster_graph(
            (
                "c1",
                &["Einstein", "relativity"],
                "Albert Einstein developed the theory of relativity, introducing the concept of spacetime.",
            ),
            (
                "c2",
                &["Einstein", "eclipse"],
                "The bending of light by gravity was confirmed during the 1919 solar eclipse, supporting Einstein's theory.",
            ),
            &["Einstein => Einstein"],
        );
        let personas = [
            persona(
                "Historian",
                "Focuses on scientific milestones and their global impact.",
            ),
            persona("Chef", "Cooks food and develops recipes."),
        ];

        let dataset = MultiHopSpecificSynthesizer::new(llm)
            .generate(&graph, &personas, 2)
            .await
            .expect("live dataset");

        assert!(!dataset.is_empty());
        for sample in dataset.iter() {
            assert!(!sample.user_input.trim().is_empty(), "empty query");
            assert!(
                sample
                    .reference
                    .as_deref()
                    .is_some_and(|r| !r.trim().is_empty()),
                "empty grounded answer"
            );
            // Spans both hops.
            assert_eq!(
                sample.reference_contexts.len(),
                2,
                "expected two hop contexts"
            );
            assert!(sample.reference_contexts[0].starts_with("<1-hop>"));
            assert!(sample.reference_contexts[1].starts_with("<2-hop>"));
            assert_eq!(
                sample.metadata.get("persona_name").map(String::as_str),
                Some("Historian"),
                "the shared entity should anchor on the historian, not the chef"
            );
            // The answer must be grounded in the source contexts — a no-op/echo generator that
            // just emitted hop tags would not reproduce these facts. (We ground-check the answer,
            // not the query: the query's style may be "misspelled", which mangles theme words.)
            let answer = sample
                .reference
                .as_deref()
                .unwrap_or_default()
                .to_lowercase();
            assert!(
                ["1919", "eclipse", "relativity", "spacetime", "gravity"]
                    .iter()
                    .any(|kw| answer.contains(kw)),
                "answer should be grounded in the contexts: {:?}",
                sample.reference
            );
            // Full synthesizer contract: synthesis type + theme + style/length metadata recorded.
            assert_eq!(
                sample.metadata.get("synthesis_type").map(String::as_str),
                Some("multi-hop")
            );
            assert_eq!(
                sample.metadata.get("themes").map(String::as_str),
                Some("Einstein")
            );
            assert!(
                sample.metadata.contains_key("query_style")
                    && sample.metadata.contains_key("query_length"),
                "missing style/length metadata: {:?}",
                sample.metadata
            );
        }
    }

    /// Build a graph whose edges (all of `rel_type`) are exactly the given `(source, target)` pairs.
    fn edge_graph(rel_type: &str, edges: &[(&str, &str)]) -> KnowledgeGraph {
        let mut ids: BTreeSet<String> = BTreeSet::new();
        for (a, b) in edges {
            ids.insert(a.to_string());
            ids.insert(b.to_string());
        }
        let mut graph = KnowledgeGraph::new();
        for id in &ids {
            graph = graph.add_node(GraphNode::new(id.clone(), "chunk"));
        }
        for (a, b) in edges {
            graph = graph.add_edge(GraphEdge::new(*a, *b, rel_type));
        }
        graph
    }

    fn cluster(ids: &[&str]) -> BTreeSet<String> {
        ids.iter().map(|id| id.to_string()).collect()
    }

    #[test]
    fn find_n_indirect_clusters_returns_superset_path_and_drops_subset() {
        // Chain a -> b -> c: the full path {a,b,c} subsumes the {b,c} subpath, so only it survives.
        let graph = edge_graph("rel", &[("a", "b"), ("b", "c")]);
        let clusters = find_n_indirect_clusters(&graph, 5, 3, false, |e| e.relationship == "rel")
            .expect("clusters");
        assert_eq!(clusters, vec![cluster(&["a", "b", "c"])]);
    }

    #[test]
    fn find_n_indirect_clusters_respects_depth_limit() {
        // depth_limit = 2 caps paths at two nodes, so the chain yields the adjacent pairs.
        let graph = edge_graph("rel", &[("a", "b"), ("b", "c"), ("c", "d")]);
        let clusters = find_n_indirect_clusters(&graph, 10, 2, false, |e| e.relationship == "rel")
            .expect("clusters");
        assert_eq!(clusters.len(), 3);
        assert!(clusters.iter().all(|c| c.len() == 2));
        assert!(clusters.contains(&cluster(&["a", "b"])));
        assert!(clusters.contains(&cluster(&["c", "d"])));
    }

    #[test]
    fn find_n_indirect_clusters_bidirectional_merges_into_longer_path() {
        // a -> b <- c: directed gives two pairs; bidirectional connects them into one 3-node path.
        let graph = edge_graph("rel", &[("a", "b"), ("c", "b")]);

        let directed = find_n_indirect_clusters(&graph, 10, 3, false, |e| e.relationship == "rel")
            .expect("directed");
        assert_eq!(directed.len(), 2);
        assert!(directed.contains(&cluster(&["a", "b"])));
        assert!(directed.contains(&cluster(&["c", "b"])));

        let bidir = find_n_indirect_clusters(&graph, 10, 3, true, |e| e.relationship == "rel")
            .expect("bidirectional");
        assert_eq!(bidir, vec![cluster(&["a", "b", "c"])]);
    }

    #[test]
    fn find_n_indirect_clusters_filters_by_relationship() {
        // Only the `summary_similarity` edge is traversed; the `other` edge is ignored.
        let graph = KnowledgeGraph::new()
            .add_node(GraphNode::new("a", "chunk"))
            .add_node(GraphNode::new("b", "chunk"))
            .add_node(GraphNode::new("c", "chunk"))
            .add_edge(GraphEdge::new("a", "b", "summary_similarity"))
            .add_edge(GraphEdge::new("b", "c", "other"));
        let clusters = find_n_indirect_clusters(&graph, 5, 3, false, |e| {
            e.relationship == "summary_similarity"
        })
        .expect("clusters");
        assert_eq!(clusters, vec![cluster(&["a", "b"])]);
    }

    #[test]
    fn find_n_indirect_clusters_caps_at_n() {
        // Three independent pairs, n = 2 -> only two clusters returned.
        let graph = edge_graph("rel", &[("a", "b"), ("c", "d"), ("e", "f")]);
        let clusters = find_n_indirect_clusters(&graph, 2, 3, false, |e| e.relationship == "rel")
            .expect("clusters");
        assert_eq!(clusters.len(), 2);
        assert!(clusters.iter().all(|c| c.len() == 2));
    }

    #[test]
    fn find_n_indirect_clusters_validates_args_and_empty_match() {
        let graph = edge_graph("rel", &[("a", "b")]);
        // depth_limit < 2.
        assert!(matches!(
            find_n_indirect_clusters(&graph, 1, 1, false, |_| true),
            Err(RagasError::Parse { .. })
        ));
        // n < 1.
        assert!(matches!(
            find_n_indirect_clusters(&graph, 0, 3, false, |_| true),
            Err(RagasError::Parse { .. })
        ));
        // No edge matches the condition.
        assert!(matches!(
            find_n_indirect_clusters(&graph, 1, 3, false, |e| e.relationship == "missing"),
            Err(RagasError::Parse { .. })
        ));
    }

    #[test]
    fn find_n_indirect_clusters_branching_yields_distinct_paths() {
        // a -> b, b -> c, b -> d: two distinct 3-node paths share the {a,b} prefix.
        let graph = edge_graph("rel", &[("a", "b"), ("b", "c"), ("b", "d")]);
        let clusters = find_n_indirect_clusters(&graph, 10, 3, false, |e| e.relationship == "rel")
            .expect("clusters");
        // Exactly the two full paths — no leftover subset/sibling clusters.
        assert_eq!(clusters.len(), 2);
        // {a,b,c} and {a,b,d} are both full paths; neither is a subset of the other.
        assert!(clusters.contains(&cluster(&["a", "b", "c"])));
        assert!(clusters.contains(&cluster(&["a", "b", "d"])));
        // The {a,b} prefix is a subset of both, so it must not appear on its own.
        assert!(!clusters.contains(&cluster(&["a", "b"])));
    }

    #[test]
    fn find_n_indirect_clusters_self_loop_is_harmless() {
        // A self-loop (a -> a) is cycle-blocked and yields no cluster; the real edge a -> b does.
        let graph = edge_graph("rel", &[("a", "a"), ("a", "b")]);
        let clusters = find_n_indirect_clusters(&graph, 5, 3, false, |e| e.relationship == "rel")
            .expect("clusters");
        assert_eq!(clusters, vec![cluster(&["a", "b"])]);
    }
}
