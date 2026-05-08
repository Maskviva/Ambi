# ambi-pipelines

Advanced cognitive execution pipelines for the [Ambi](https://github.com/Maskviva/Ambi) AI agent framework.

## Pipelines

| Pipeline | Description |
|---|---|
| **RAG** | Retrieval-Augmented Generation — fetches documents, packs them into context respecting token limits, then delegates to the inner chat loop. |
| **Chain-of-Thought** | Runs multiple independent reasoning branches in parallel, extracts answers, and selects the most frequent via majority voting (Self-Consistency). |
| **Tree-of-Thoughts** | Breadth-First beam search — expands, evaluates, and prunes thought branches at each depth step. |
| **Reflexion** | Actor-Evaluator loop — generates a response, evaluates it, writes critiques to persistent KV memory, and retries. |

## Usage

```toml
[dependencies]
ambi-pipelines = "0.1"
```

### RAG

```rust
use ambi_pipelines::rag::{StandardRagPipeline, Retriever};

struct MyRetriever;
#[async_trait]
impl Retriever for MyRetriever { /* ... */ }

let pipeline = StandardRagPipeline::create(MyRetriever)
    .max_context_tokens(4096)
    .concurrency(5);
```

### Chain-of-Thought (Self-Consistency)

```rust
use ambi_pipelines::cot::SelfConsistencyPipeline;
use ambi_pipelines::cot::PatternExtractor;

let pipeline = SelfConsistencyPipeline::create(
    PatternExtractor::new("Answer:", "\n")
)
.branches(5)
.concurrency(5);
```

### Tree-of-Thoughts (BFS Beam Search)

```rust
use ambi_pipelines::tot::BfsBeamSearchPipeline;
use ambi_pipelines::tot::{TotExpander, TotEvaluator};

let pipeline = BfsBeamSearchPipeline::create(my_expander, my_evaluator)
    .branching_factor(3)
    .beam_width(2)
    .max_steps(5);
```

### Reflexion

```rust
use ambi_pipelines::reflexion::ReflexionPipeline;
use ambi_memory::InMemoryKvProvider;

let pipeline = ReflexionPipeline::create(my_evaluator, InMemoryKvProvider::new())
    .max_retries(3);
```

## License

Apache-2.0
