# ambi-pipelines

Advanced cognitive execution pipelines for the Ambi AI agent framework.

```toml
[dependencies]
ambi-pipelines = "0.1"
```

All pipelines in this crate implement the `Pipeline` trait, making them drop-in replacements for the default `ChatRunner`:

```rust
use ambi_pipelines::rag::StandardRagPipeline;

let pipeline = StandardRagPipeline::create(my_retriever);

// Use anywhere a Pipeline is expected
let reply = pipeline.execute(&agent, &state, input).await?;
```

## RAG — Retrieval-Augmented Generation

Retrieves relevant documents, token-aware packs them into context, then delegates to the inner chat loop.

```rust
use ambi_pipelines::rag::{StandardRagPipeline, Retriever};
use async_trait::async_trait;

struct MyRetriever;

#[async_trait]
impl Retriever for MyRetriever {
    async fn retrieve(&self, query: &str)
        -> Result<Vec<Document>, Box<dyn Error + Send + Sync>>
    {
        // Query your vector database, search engine, or file system
        Ok(vec![Document::new("doc-1", "Ambi is an AI framework.", 0.95)])
    }
}

let pipeline = StandardRagPipeline::create(MyRetriever)
    .max_context_tokens(4096)
    .concurrency(5);

let reply = pipeline.execute(&agent, &state, input).await?;
```

The `ContextPacker` sorts documents by descending score and stops before exceeding the token budget, attaching citation IDs for LLM traceability.

### SemanticMemoryRetriever

A pre-built `Retriever` that wraps `ambi-memory`'s `SemanticMemoryProvider`:

```rust
use ambi_pipelines::rag::SemanticMemoryRetriever;
use ambi_memory::SemanticMemoryProvider;

let retriever = SemanticMemoryRetriever::new(my_memory_provider, "session-1", 5);
let pipeline = StandardRagPipeline::create(retriever);
```

## Chain-of-Thought Self-Consistency

Runs multiple independent reasoning branches in parallel, extracts a canonical answer from each, and picks the winner via majority voting.

```rust
use ambi_pipelines::cot::SelfConsistencyPipeline;
use ambi_pipelines::cot::PatternExtractor;

let pipeline = SelfConsistencyPipeline::create(
    PatternExtractor::new("Answer:", "\n")
)
.branches(5)        // 5 parallel reasoning paths
.concurrency(5);    // inner runner concurrency
```

### Custom extractors and aggregators

```rust
use ambi_pipelines::cot::{AnswerExtractor, Aggregator};

struct MyExtractor;
impl AnswerExtractor for MyExtractor {
    fn extract(&self, raw: &str) -> Option<String> { /* ... */ }
}

struct MyAggregator;
impl Aggregator for MyAggregator {
    fn aggregate(&self, answers: &[Option<String>]) -> Option<(usize, String)> { /* ... */ }
}

let pipeline = SelfConsistencyPipeline::create(MyExtractor)
    .aggregator(MyAggregator);
```

## Tree-of-Thoughts BFS Beam Search

Explores multi-step reasoning via breadth-first beam search — expands candidate thoughts, evaluates them, prunes low-scoring branches, and keeps the top `B` nodes.

```rust
use ambi_pipelines::tot::BfsBeamSearchPipeline;
use ambi_pipelines::tot::{TotExpander, TotEvaluator, Evaluation};

struct MyExpander;
#[async_trait]
impl TotExpander for MyExpander {
    async fn expand(&self, agent: &Agent, state: &Arc<RwLock<AgentState>>,
        runner: &ChatRunner, k: usize) -> Result<Vec<String>> { /* ... */ }
}

struct MyEvaluator;
#[async_trait]
impl TotEvaluator for MyEvaluator {
    async fn evaluate(&self, agent: &Agent, state: &Arc<RwLock<AgentState>>,
        runner: &ChatRunner, thought: &str) -> Result<Evaluation> { /* ... */ }
}

let pipeline = BfsBeamSearchPipeline::create(MyExpander, MyEvaluator)
    .branching_factor(3)   // K thoughts per node
    .beam_width(2)         // B nodes kept per depth
    .max_steps(5);         // max exploration depth
```

## Reflexion — Actor-Evaluator loop

Generates a response, evaluates it against a quality bar, writes critiques to persistent KV memory, and retries until passing or max retries exhausted.

```rust
use ambi_pipelines::reflexion::ReflexionPipeline;
use ambi_pipelines::reflexion::{Evaluator, EvaluationResult};
use ambi_memory::InMemoryKvProvider;

struct MyEvaluator;
#[async_trait]
impl Evaluator for MyEvaluator {
    async fn evaluate(&self, response: &str)
        -> Result<EvaluationResult, Box<dyn Error + Send + Sync>>
    {
        if response.contains("I don't know") {
            Ok(EvaluationResult { is_pass: false, critique: Some("Provide a definite answer.".into()) })
        } else {
            Ok(EvaluationResult { is_pass: true, critique: None })
        }
    }
}

let pipeline = ReflexionPipeline::create(MyEvaluator, InMemoryKvProvider::new())
    .max_retries(3);
```
