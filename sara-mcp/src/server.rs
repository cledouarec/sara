//! The MCP protocol handler serving a Sara knowledge graph.

use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use sara_core::graph::{KnowledgeGraph, search};
use sara_core::model::ItemType;
use sara_core::schema;
use sara_core::service::{load_graph, read_body};
use tokio::sync::RwLock;
use tokio::task::spawn_blocking;

use crate::ServerOptions;
use crate::params::{SEARCH_LIMIT, SearchItemsParams, parse_item_types, type_list};
use crate::views::SearchResults;

/// Name the server announces to MCP clients.
const SERVER_NAME: &str = "sara";

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
             Read sara_describe_model for the fields and relations of each type.",
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
