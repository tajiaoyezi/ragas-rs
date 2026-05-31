use ragas::{
    GraphNode, KnowledgeGraph, PersonaGenerator, build_chunk_relationships, split_text_into_chunks,
    synthesize_single_hop_sample,
};

fn main() {
    let chunks = split_text_into_chunks(
        "doc-1",
        "ragas-rs creates grounded samples from source text for RAG evaluation.",
        80,
    );
    let graph = chunks.iter().fold(
        KnowledgeGraph::new().add_node(GraphNode::new("doc-1", "document")),
        |graph, chunk| graph.add_node(chunk.to_graph_node()),
    );
    let graph = build_chunk_relationships(graph, "doc-1", &chunks);
    let persona = PersonaGenerator::new("example").generate(
        "QA Engineer",
        "evaluation engineer",
        vec!["ask grounded questions".to_string()],
    );
    let sample =
        synthesize_single_hop_sample(&graph, &chunks[0].id, &persona).expect("single-hop sample");

    println!("{}", sample.sample.user_input);
}
