use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    ChatMessage, EmbeddingProvider, EmbeddingRequest, EvaluationDataset, LlmProvider, LlmRequest,
    RagasError, SingleTurnSample, cosine_similarity,
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
}
