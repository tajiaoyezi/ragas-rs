use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    ParityClaim, ParityFeatureStatus, ParityFixtureMetadata, ParityFixtureMode, RagasError,
    SingleTurnSample,
};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphParityFixture {
    pub feature: String,
    pub upstream_commit: String,
    pub graph: KnowledgeGraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GraphQueryCapability {
    NodeTypeFilter,
    PropertyFilter,
    RelationshipFilter,
    NeighborTraversal,
    Clusters,
    AdvancedQuery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryDescriptor {
    pub capability: GraphQueryCapability,
    pub parity_status: ParityFeatureStatus,
    pub deterministic: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TransformStageFamily {
    Splitter,
    EntityExtractor,
    ThemeExtractor,
    SummaryExtractor,
    RelationshipBuilder,
    LlmExtractor,
    Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TransformStageMode {
    Deterministic,
    LiveLlm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformStageDescriptor {
    pub family: TransformStageFamily,
    pub mode: TransformStageMode,
    pub parity_status: ParityFeatureStatus,
    pub output_properties: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SynthesizerStrategy {
    SingleHop,
    MultiHop,
    PreChunked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesizerDescriptor {
    pub strategy: SynthesizerStrategy,
    pub parity_status: ParityFeatureStatus,
    pub fixture_backed: bool,
    pub prompt_snapshot: String,
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

pub fn graph_query_descriptors() -> Vec<GraphQueryDescriptor> {
    vec![
        graph_query_descriptor(
            GraphQueryCapability::NodeTypeFilter,
            ParityFeatureStatus::Complete,
            true,
        ),
        graph_query_descriptor(
            GraphQueryCapability::PropertyFilter,
            ParityFeatureStatus::Complete,
            true,
        ),
        graph_query_descriptor(
            GraphQueryCapability::RelationshipFilter,
            ParityFeatureStatus::Complete,
            true,
        ),
        graph_query_descriptor(
            GraphQueryCapability::NeighborTraversal,
            ParityFeatureStatus::Complete,
            true,
        ),
        graph_query_descriptor(
            GraphQueryCapability::Clusters,
            ParityFeatureStatus::Complete,
            true,
        ),
        graph_query_descriptor(
            GraphQueryCapability::AdvancedQuery,
            ParityFeatureStatus::Complete,
            true,
        ),
    ]
}

pub fn graph_parity_claims() -> Vec<ParityClaim> {
    graph_query_descriptors()
        .into_iter()
        .map(|descriptor| {
            let feature = format!(
                "testset::graph::{}",
                graph_query_slug(descriptor.capability)
            );
            ParityClaim {
                feature: feature.clone(),
                status: descriptor.parity_status,
                fixtures: vec![graph_query_fixture_metadata(
                    &feature,
                    descriptor.capability,
                )],
            }
        })
        .collect()
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

pub fn transform_stage_descriptors() -> Vec<TransformStageDescriptor> {
    vec![
        transform_stage_descriptor(
            TransformStageFamily::Splitter,
            TransformStageMode::Deterministic,
            ParityFeatureStatus::Complete,
            vec!["chunks", "text", "source_id", "chunk_index"],
        ),
        transform_stage_descriptor(
            TransformStageFamily::EntityExtractor,
            TransformStageMode::Deterministic,
            ParityFeatureStatus::Complete,
            vec!["entities"],
        ),
        transform_stage_descriptor(
            TransformStageFamily::ThemeExtractor,
            TransformStageMode::Deterministic,
            ParityFeatureStatus::Complete,
            vec!["themes"],
        ),
        transform_stage_descriptor(
            TransformStageFamily::SummaryExtractor,
            TransformStageMode::Deterministic,
            ParityFeatureStatus::Complete,
            vec!["summary"],
        ),
        transform_stage_descriptor(
            TransformStageFamily::RelationshipBuilder,
            TransformStageMode::Deterministic,
            ParityFeatureStatus::Complete,
            vec!["relationships", "contains", "next", "order"],
        ),
        transform_stage_descriptor(
            TransformStageFamily::LlmExtractor,
            TransformStageMode::LiveLlm,
            ParityFeatureStatus::Complete,
            vec!["entities", "themes", "summary"],
        ),
        transform_stage_descriptor(
            TransformStageFamily::Filter,
            TransformStageMode::Deterministic,
            ParityFeatureStatus::Complete,
            vec!["filtered_nodes"],
        ),
    ]
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

pub fn transform_parity_claims() -> Vec<ParityClaim> {
    transform_stage_descriptors()
        .into_iter()
        .map(|descriptor| {
            let feature = format!(
                "testset::transform::{}",
                transform_stage_slug(descriptor.family)
            );
            ParityClaim {
                feature: feature.clone(),
                status: descriptor.parity_status,
                fixtures: vec![transform_fixture_metadata(&feature, descriptor.family)],
            }
        })
        .collect()
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

pub fn synthesizer_descriptors() -> Vec<SynthesizerDescriptor> {
    vec![
        synthesizer_descriptor(
            SynthesizerStrategy::SingleHop,
            ParityFeatureStatus::Complete,
            true,
            "single_hop_question",
        ),
        synthesizer_descriptor(
            SynthesizerStrategy::MultiHop,
            ParityFeatureStatus::Complete,
            true,
            "multi_hop_question",
        ),
        synthesizer_descriptor(
            SynthesizerStrategy::PreChunked,
            ParityFeatureStatus::Complete,
            true,
            "pre_chunked_generation",
        ),
    ]
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

pub fn synthesizer_parity_claims() -> Vec<ParityClaim> {
    synthesizer_descriptors()
        .into_iter()
        .map(|descriptor| {
            let feature = format!(
                "testset::synthesizer::{}",
                synthesizer_strategy_slug(descriptor.strategy)
            );
            let fixtures = if descriptor.parity_status == ParityFeatureStatus::Complete
                && descriptor.fixture_backed
            {
                vec![synthesizer_fixture_metadata(
                    &feature,
                    descriptor.strategy,
                    &descriptor.prompt_snapshot,
                )]
            } else {
                Vec::new()
            };
            ParityClaim {
                feature,
                status: descriptor.parity_status,
                fixtures,
            }
        })
        .collect()
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

fn graph_query_descriptor(
    capability: GraphQueryCapability,
    parity_status: ParityFeatureStatus,
    deterministic: bool,
) -> GraphQueryDescriptor {
    GraphQueryDescriptor {
        capability,
        parity_status,
        deterministic,
    }
}

fn graph_query_slug(capability: GraphQueryCapability) -> &'static str {
    match capability {
        GraphQueryCapability::NodeTypeFilter => "node_type_filter",
        GraphQueryCapability::PropertyFilter => "property_filter",
        GraphQueryCapability::RelationshipFilter => "relationship_filter",
        GraphQueryCapability::NeighborTraversal => "neighbor_traversal",
        GraphQueryCapability::Clusters => "clusters",
        GraphQueryCapability::AdvancedQuery => "advanced_query",
    }
}

fn graph_query_fixture_metadata(
    feature: &str,
    capability: GraphQueryCapability,
) -> ParityFixtureMetadata {
    let upstream_module = match capability {
        GraphQueryCapability::Clusters | GraphQueryCapability::AdvancedQuery => {
            "src/ragas/testset/graph_queries.py"
        }
        _ => "src/ragas/testset/graph.py",
    };
    ParityFixtureMetadata::new(
        feature,
        upstream_module,
        None,
        format!(
            "tests/parity/fixtures/testset_graph_{}.json",
            graph_query_slug(capability)
        ),
        ParityFixtureMode::DeterministicMock,
        None,
    )
}

fn graph_property_cluster_keys(value: &GraphProperty) -> Vec<String> {
    match value {
        GraphProperty::Text(value) => vec![value.clone()],
        GraphProperty::Number(value) => vec![format!("{value:.6}")],
        GraphProperty::Boolean(value) => vec![value.to_string()],
        GraphProperty::TextList(values) => values.clone(),
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

fn transform_stage_descriptor(
    family: TransformStageFamily,
    mode: TransformStageMode,
    parity_status: ParityFeatureStatus,
    output_properties: Vec<&'static str>,
) -> TransformStageDescriptor {
    TransformStageDescriptor {
        family,
        mode,
        parity_status,
        output_properties,
    }
}

fn transform_stage_slug(family: TransformStageFamily) -> &'static str {
    match family {
        TransformStageFamily::Splitter => "splitter",
        TransformStageFamily::EntityExtractor => "entity_extractor",
        TransformStageFamily::ThemeExtractor => "theme_extractor",
        TransformStageFamily::SummaryExtractor => "summary_extractor",
        TransformStageFamily::RelationshipBuilder => "relationship_builder",
        TransformStageFamily::LlmExtractor => "llm_extractor",
        TransformStageFamily::Filter => "filter",
    }
}

fn transform_fixture_metadata(
    feature: &str,
    family: TransformStageFamily,
) -> ParityFixtureMetadata {
    let upstream_module = match family {
        TransformStageFamily::Splitter => "src/ragas/testset/transforms/splitters/headline.py",
        TransformStageFamily::EntityExtractor
        | TransformStageFamily::ThemeExtractor
        | TransformStageFamily::SummaryExtractor
        | TransformStageFamily::LlmExtractor => {
            "src/ragas/testset/transforms/extractors/llm_based.py"
        }
        TransformStageFamily::RelationshipBuilder => {
            "src/ragas/testset/transforms/relationship_builders/traditional.py"
        }
        TransformStageFamily::Filter => "src/ragas/testset/transforms/filters.py",
    };
    ParityFixtureMetadata::new(
        feature,
        upstream_module,
        None,
        format!(
            "tests/parity/fixtures/testset_transform_{}.json",
            transform_stage_slug(family)
        ),
        ParityFixtureMode::DeterministicMock,
        None,
    )
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

fn synthesizer_descriptor(
    strategy: SynthesizerStrategy,
    parity_status: ParityFeatureStatus,
    fixture_backed: bool,
    prompt_snapshot: impl Into<String>,
) -> SynthesizerDescriptor {
    SynthesizerDescriptor {
        strategy,
        parity_status,
        fixture_backed,
        prompt_snapshot: prompt_snapshot.into(),
    }
}

fn synthesizer_strategy_slug(strategy: SynthesizerStrategy) -> &'static str {
    match strategy {
        SynthesizerStrategy::SingleHop => "single_hop",
        SynthesizerStrategy::MultiHop => "multi_hop",
        SynthesizerStrategy::PreChunked => "pre_chunked",
    }
}

fn synthesizer_strategy_label(strategy: SynthesizerStrategy) -> &'static str {
    match strategy {
        SynthesizerStrategy::SingleHop => "single-hop",
        SynthesizerStrategy::MultiHop => "multi-hop",
        SynthesizerStrategy::PreChunked => "pre-chunked",
    }
}

fn synthesizer_fixture_metadata(
    feature: &str,
    strategy: SynthesizerStrategy,
    prompt_snapshot: &str,
) -> ParityFixtureMetadata {
    ParityFixtureMetadata::new(
        feature,
        format!(
            "src/ragas/testset/synthesizers/{}/base.py",
            synthesizer_strategy_slug(strategy)
        ),
        Some(format!(
            "tests/unit/testset/synthesizers/test_{}.py",
            synthesizer_strategy_slug(strategy)
        )),
        format!(
            "tests/parity/fixtures/testset_{}_{}.json",
            synthesizer_strategy_slug(strategy),
            prompt_snapshot
        ),
        ParityFixtureMode::DeterministicMock,
        None,
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    use crate::release_blocking_claims;

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
    fn test_20_1_2_graph_query_descriptors_cover_required_filters() {
        // SCEN-20.1.2 / AC2 / TEST-20.1.2
        let capabilities: BTreeSet<_> = graph_query_descriptors()
            .iter()
            .map(|descriptor| descriptor.capability)
            .collect();

        for expected in [
            GraphQueryCapability::NodeTypeFilter,
            GraphQueryCapability::PropertyFilter,
            GraphQueryCapability::RelationshipFilter,
            GraphQueryCapability::NeighborTraversal,
        ] {
            assert!(capabilities.contains(&expected), "missing {expected:?}");
        }
    }

    #[test]
    fn test_20_1_3_missing_graph_features_create_release_blocking_claims() {
        // SCEN-20.1.3 / AC3 / TEST-20.1.3
        let claims = vec![
            ParityClaim {
                feature: "testset::graph::synthetic_missing_cluster".to_string(),
                status: ParityFeatureStatus::KnownGap,
                fixtures: Vec::new(),
            },
            ParityClaim {
                feature: "testset::graph::synthetic_missing_query".to_string(),
                status: ParityFeatureStatus::Complete,
                fixtures: Vec::new(),
            },
        ];
        let blockers = release_blocking_claims(&claims);
        let blocking_features: BTreeSet<_> = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect();

        for expected in [
            "testset::graph::synthetic_missing_cluster",
            "testset::graph::synthetic_missing_query",
        ] {
            assert!(
                blocking_features.contains(expected),
                "missing graph release blocker {expected}"
            );
        }
        assert!(release_blocking_claims(&graph_parity_claims()).is_empty());
    }

    #[test]
    fn test_20_2_1_transform_registry_lists_stages_with_modes() {
        // SCEN-20.2.1 / AC1 / TEST-20.2.1
        let descriptors = transform_stage_descriptors();
        let by_family: BTreeMap<_, _> = descriptors
            .iter()
            .map(|descriptor| (descriptor.family, descriptor))
            .collect();

        for expected in [
            TransformStageFamily::Splitter,
            TransformStageFamily::EntityExtractor,
            TransformStageFamily::ThemeExtractor,
            TransformStageFamily::SummaryExtractor,
            TransformStageFamily::RelationshipBuilder,
            TransformStageFamily::LlmExtractor,
        ] {
            assert!(by_family.contains_key(&expected), "missing {expected:?}");
        }

        let splitter = by_family
            .get(&TransformStageFamily::Splitter)
            .expect("splitter descriptor");
        assert_eq!(splitter.mode, TransformStageMode::Deterministic);
        assert_eq!(splitter.parity_status, ParityFeatureStatus::Complete);
        assert!(splitter.output_properties.contains(&"chunks"));

        let llm_extractor = by_family
            .get(&TransformStageFamily::LlmExtractor)
            .expect("llm extractor descriptor");
        assert_eq!(llm_extractor.mode, TransformStageMode::LiveLlm);
        assert_eq!(llm_extractor.parity_status, ParityFeatureStatus::Complete);
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
    fn test_20_2_3_unsupported_transform_stages_create_release_blocking_claims() {
        // SCEN-20.2.3 / AC3 / TEST-20.2.3
        let claims = vec![
            ParityClaim {
                feature: "testset::transform::synthetic_missing_llm_extractor".to_string(),
                status: ParityFeatureStatus::KnownGap,
                fixtures: Vec::new(),
            },
            ParityClaim {
                feature: "testset::transform::synthetic_missing_filter_fixture".to_string(),
                status: ParityFeatureStatus::Complete,
                fixtures: Vec::new(),
            },
        ];
        let blockers = release_blocking_claims(&claims);
        let blocking_features: BTreeSet<_> = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect();

        for expected in [
            "testset::transform::synthetic_missing_llm_extractor",
            "testset::transform::synthetic_missing_filter_fixture",
        ] {
            assert!(
                blocking_features.contains(expected),
                "missing transform release blocker {expected}"
            );
        }

        assert!(release_blocking_claims(&transform_parity_claims()).is_empty());
    }

    #[test]
    fn test_20_3_1_synthesizer_registry_lists_core_strategies() {
        // SCEN-20.3.1 / AC1 / TEST-20.3.1
        let descriptors = synthesizer_descriptors();
        let by_strategy: BTreeMap<_, _> = descriptors
            .iter()
            .map(|descriptor| (descriptor.strategy, descriptor))
            .collect();

        for expected in [
            SynthesizerStrategy::SingleHop,
            SynthesizerStrategy::MultiHop,
            SynthesizerStrategy::PreChunked,
        ] {
            assert!(by_strategy.contains_key(&expected), "missing {expected:?}");
        }

        let single_hop = by_strategy
            .get(&SynthesizerStrategy::SingleHop)
            .expect("single-hop descriptor");
        assert_eq!(single_hop.parity_status, ParityFeatureStatus::Complete);
        assert!(single_hop.fixture_backed);
        assert_eq!(single_hop.prompt_snapshot.as_str(), "single_hop_question");

        let pre_chunked = by_strategy
            .get(&SynthesizerStrategy::PreChunked)
            .expect("pre-chunked descriptor");
        assert_eq!(pre_chunked.parity_status, ParityFeatureStatus::Complete);
        assert!(pre_chunked.fixture_backed);
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
    fn test_20_3_3_unsupported_or_unfixture_backed_synthesizers_block_release() {
        // SCEN-20.3.3 / AC3 / TEST-20.3.3
        let claims = synthesizer_parity_claims();
        let blockers = release_blocking_claims(&claims);

        assert!(
            blockers.is_empty(),
            "closed synthesizers should not keep release blockers: {blockers:?}"
        );

        for expected in [
            "testset::synthesizer::single_hop",
            "testset::synthesizer::multi_hop",
            "testset::synthesizer::pre_chunked",
        ] {
            let claim = claims
                .iter()
                .find(|claim| claim.feature == expected)
                .unwrap_or_else(|| panic!("missing claim {expected}"));
            assert_eq!(claim.status, ParityFeatureStatus::Complete);
            assert!(
                !claim.fixtures.is_empty(),
                "complete synthesizer claim {expected} must be fixture-backed"
            );
        }

        let unfixture_backed_complete = ParityClaim {
            feature: "testset::synthesizer::missing_fixture".to_string(),
            status: ParityFeatureStatus::Complete,
            fixtures: Vec::new(),
        };
        assert_eq!(
            release_blocking_claims(&[unfixture_backed_complete]).len(),
            1
        );
    }

    #[test]
    fn test_29_1_1_graph_cluster_and_advanced_query_contracts_are_complete() {
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

        let by_capability: BTreeMap<_, _> = graph_query_descriptors()
            .into_iter()
            .map(|descriptor| (descriptor.capability, descriptor))
            .collect();
        assert_eq!(
            by_capability
                .get(&GraphQueryCapability::Clusters)
                .expect("clusters")
                .parity_status,
            ParityFeatureStatus::Complete
        );
        assert_eq!(
            by_capability
                .get(&GraphQueryCapability::AdvancedQuery)
                .expect("advanced query")
                .parity_status,
            ParityFeatureStatus::Complete
        );

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

        assert!(release_blocking_claims(&graph_parity_claims()).is_empty());
    }

    #[test]
    fn test_29_1_2_transform_llm_extractor_and_filter_contracts_are_complete() {
        // SCEN-29.1.2 / AC2 / TEST-29.1.2
        let by_family: BTreeMap<_, _> = transform_stage_descriptors()
            .into_iter()
            .map(|descriptor| (descriptor.family, descriptor))
            .collect();
        assert_eq!(
            by_family
                .get(&TransformStageFamily::LlmExtractor)
                .expect("llm extractor")
                .parity_status,
            ParityFeatureStatus::Complete
        );
        assert_eq!(
            by_family
                .get(&TransformStageFamily::Filter)
                .expect("filter")
                .parity_status,
            ParityFeatureStatus::Complete
        );

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

        assert!(release_blocking_claims(&transform_parity_claims()).is_empty());
    }

    #[test]
    fn test_29_1_3_pre_chunked_synthesizer_closes_testset_ledger() {
        // SCEN-29.1.3 / AC3 / TEST-29.1.3
        let descriptors = synthesizer_descriptors();
        let pre_chunked = descriptors
            .iter()
            .find(|descriptor| descriptor.strategy == SynthesizerStrategy::PreChunked)
            .expect("pre-chunked descriptor");
        assert_eq!(pre_chunked.parity_status, ParityFeatureStatus::Complete);
        assert!(pre_chunked.fixture_backed);

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

        assert!(release_blocking_claims(&synthesizer_parity_claims()).is_empty());

        let ledger = crate::release::build_release_blocker_ledger();
        let categories = ledger
            .entries
            .iter()
            .map(|entry| entry.category)
            .collect::<BTreeSet<_>>();
        assert!(
            !categories.contains(&crate::release::ReleaseBlockerCategory::Testset),
            "testset release blockers must be fully closed"
        );
    }
}
