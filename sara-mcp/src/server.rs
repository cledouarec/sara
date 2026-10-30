//! The MCP protocol handler serving a Sara knowledge graph.

use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use sara_core::graph::{KnowledgeGraph, LookupResult, TraversalDirection, impact, reach, search};
use sara_core::model::{Item, ItemSummary, ItemType};
use sara_core::schema;
use sara_core::service::{load_graph, read_body};
use tokio::sync::RwLock;
use tokio::task::spawn_blocking;

use crate::ServerOptions;
use crate::params::{
    AnalyzeImpactParams, GetItemParams, IMPACT_LIMIT, MAX_IMPACT_ORIGINS, SEARCH_LIMIT,
    SearchItemsParams, TRACE_LIMIT, TraceDirection, TraceItemParams, parse_item_types, type_list,
};
use crate::views::{ImpactView, ItemView, SearchResults, TraceView};

/// Name the server announces to MCP clients.
const SERVER_NAME: &str = "sara";

/// Hint appended to the errors about an unknown item.
const SEARCH_HINT: &str = "Use sara_search_items to find items by their content.";

/// Protocol handler serving a Sara knowledge graph to MCP clients.
///
/// Clones are cheap and share the same settings and lock.
#[derive(Debug, Clone)]
pub struct SaraServer {
    options: Arc<ServerOptions>,
    /// Held shared while a request reads the files and exclusively while
    /// one writes them, so no request sees a write half done.
    lock: Arc<RwLock<()>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl SaraServer {
    /// Creates a server for the repositories and settings in `options`.
    #[must_use]
    pub fn new(options: ServerOptions) -> Self {
        Self {
            options: Arc::new(options),
            lock: Arc::new(RwLock::new(())),
            tool_router: Self::tool_router(),
        }
    }

    /// Describes the active model: each item type with its ID prefix and
    /// format, fields and allowed relations, then the relations and how
    /// many items of each type the graph holds. Read it before creating or
    /// editing items.
    #[tool(
        name = "sara_describe_model",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn describe_model(&self) -> Result<String, String> {
        let counts = self.read_graph(|graph| Ok(graph.count_by_type())).await?;

        let mut text = schema::active().to_yaml().map_err(|e| e.to_string())?;
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str("item_counts:\n");
        for item_type in ItemType::all() {
            let count = counts.get(&item_type).copied().unwrap_or(0);
            let _ = writeln!(text, "  {}: {count}", item_type.as_str());
        }
        Ok(text)
    }

    /// Returns one item: its type, name, description, source file, field
    /// values, the relations its file declares (`relations`, in the form
    /// sara_update_item takes), its direct links in both directions (`links`)
    /// and, unless `include_body` is false, its Markdown body. A linked item
    /// missing from `relations` is linked from its own file.
    #[tool(
        name = "sara_get_item",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_item(
        &self,
        Parameters(params): Parameters<GetItemParams>,
    ) -> Result<Json<ItemView>, String> {
        self.read_graph(move |graph| {
            let item = find_item(graph, &params.id)?;
            let body = if params.include_body {
                Some(read_body(item).map_err(|e| e.to_string())?)
            } else {
                None
            };
            Ok(Json(ItemView::new(graph, item, body)))
        })
        .await
    }

    /// Searches items by the words of `query` in their id, name,
    /// description, fields and Markdown body, best matches first; with an
    /// empty query, lists items by id. Each word counts once, for the most
    /// significant place it appears in, and items holding more of the words
    /// rank first. Hits are compact and paged: read an item in full with
    /// sara_get_item.
    #[tool(
        name = "sara_search_items",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn search_items(
        &self,
        Parameters(params): Parameters<SearchItemsParams>,
    ) -> Result<Json<SearchResults>, String> {
        let types = parse_item_types(&params.types)?;
        let limit = SEARCH_LIMIT.resolve(params.limit);

        self.read_graph(move |graph| {
            let hits = search(graph, &params.query, &types, |item| read_body(item).ok());
            Ok(Json(SearchResults::page(hits, params.offset, limit)))
        })
        .await
    }

    /// Follows traceability links transitively from an item: `upstream`
    /// toward the solution (the items it refines, derives from or
    /// satisfies, and theirs), `downstream` toward detailed designs (the
    /// items refining, deriving from or satisfying it, and theirs), or both.
    /// Each reached item appears once, at its shortest distance, with every
    /// link followed, so shared parents are not repeated. Peer links such as
    /// depends_on are not followed: see sara_analyze_impact.
    #[tool(
        name = "sara_trace_item",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn trace_item(
        &self,
        Parameters(params): Parameters<TraceItemParams>,
    ) -> Result<Json<TraceView>, String> {
        let limit = TRACE_LIMIT.resolve(params.limit);

        self.read_graph(move |graph| {
            let item = find_item(graph, &params.id)?;
            let origins = [item.id.clone()];
            let chain = |direction| {
                let mut reached = reach(graph, &origins, direction, params.max_depth);
                reached.truncate(limit);
                reached
            };
            let wants = |direction| {
                params.direction == direction || params.direction == TraceDirection::Both
            };

            Ok(Json(TraceView {
                origin: ItemSummary::from(item),
                upstream: wants(TraceDirection::Upstream)
                    .then(|| chain(TraversalDirection::Upstream)),
                downstream: wants(TraceDirection::Downstream)
                    .then(|| chain(TraversalDirection::Downstream)),
            }))
        })
        .await
    }

    /// Lists the items a change to the given items may impact: every item
    /// tracing down from them (refining, deriving from, satisfying or
    /// justified by them, and so on) and every item declaring a dependency
    /// on them (depends_on, supersedes), transitively. Each impacted item
    /// comes with its file and the link through which the change reaches
    /// it. Call it before changing an item to know what to review.
    #[tool(
        name = "sara_analyze_impact",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn analyze_impact(
        &self,
        Parameters(params): Parameters<AnalyzeImpactParams>,
    ) -> Result<Json<ImpactView>, String> {
        if params.ids.is_empty() {
            return Err("Give the ids of the items about to change.".to_string());
        }
        if params.ids.len() > MAX_IMPACT_ORIGINS {
            return Err(format!(
                "At most {MAX_IMPACT_ORIGINS} items can be analyzed at once; got {}.",
                params.ids.len()
            ));
        }
        let limit = IMPACT_LIMIT.resolve(params.limit);

        self.read_graph(move |graph| {
            let changed = params
                .ids
                .iter()
                .map(|id| find_item(graph, id))
                .collect::<Result<Vec<&Item>, String>>()?;
            let origins: Vec<_> = changed.iter().map(|item| item.id.clone()).collect();
            let reached = impact(graph, &origins, params.max_depth);
            let origins = changed.into_iter().map(ItemSummary::from).collect();
            Ok(Json(ImpactView::new(graph, origins, reached, limit)))
        })
        .await
    }

    /// Runs `query` on the graph freshly loaded from the repositories, while
    /// other requests may read the files too.
    async fn read_graph<T: Send + 'static>(
        &self,
        query: impl FnOnce(&KnowledgeGraph) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        let _guard = self.lock.read().await;
        self.with_graph(query).await
    }

    /// Loads the graph from the repositories and runs `task` on it; the
    /// caller holds the lock.
    async fn with_graph<T: Send + 'static>(
        &self,
        task: impl FnOnce(&KnowledgeGraph) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        self.on_repositories(move |repositories| {
            let (graph, _) = load_graph(repositories).map_err(|e| e.to_string())?;
            task(&graph)
        })
        .await
    }

    /// Runs `task` on the configured repositories on a blocking thread; the
    /// caller holds the lock. A failure, including a panic in `task`,
    /// becomes an error message for the client.
    async fn on_repositories<T: Send + 'static>(
        &self,
        task: impl FnOnce(&[PathBuf]) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        let repositories = self.options.repositories.clone();
        spawn_blocking(move || task(&repositories))
            .await
            .map_err(|e| format!("internal error: {e}"))?
    }

    /// Returns the instructions sent to clients when they connect: what the
    /// graph holds, its item types and where its files live.
    fn instructions(&self) -> String {
        let repositories: Vec<String> = self
            .options
            .repositories
            .iter()
            .map(|path| path.display().to_string())
            .collect();

        format!(
            "Sara manages architecture documents and requirements as a knowledge graph. \
             Each Markdown file whose YAML frontmatter declares an id, a type and a name \
             is an item, linked to other items by traceability relations.\n\n\
             Item types (id and ID prefix): {}.\n\
             Repositories holding the items: {}.\n\n\
             Find items by their content with sara_search_items. \
             Read sara_describe_model for the fields and relations of each type. \
             Read one item, its links and its body with sara_get_item. \
             Follow its traceability chains with sara_trace_item. \
             Before changing an item, list what it impacts with sara_analyze_impact.",
            type_list(),
            repositories.join(", ")
        )
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SaraServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(SERVER_NAME, env!("CARGO_PKG_VERSION")))
            .with_instructions(self.instructions())
    }
}

/// Looks up an item by id, or explains how to find the right one.
pub(crate) fn find_item<'a>(graph: &'a KnowledgeGraph, id: &str) -> Result<&'a Item, String> {
    match graph.lookup(id) {
        LookupResult::Found(item) => Ok(item),
        LookupResult::NotFound { suggestions } if suggestions.is_empty() => {
            Err(format!("No item has the id '{id}'. {SEARCH_HINT}"))
        }
        LookupResult::NotFound { suggestions } => {
            let suggestions: Vec<&str> = suggestions.iter().map(|id| id.as_str()).collect();
            Err(format!(
                "No item has the id '{id}'; did you mean {}? {SEARCH_HINT}",
                suggestions.join(", ")
            ))
        }
    }
}
