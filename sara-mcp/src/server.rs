//! The MCP protocol handler serving a Sara knowledge graph.

use std::env;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use sara_core::graph::{KnowledgeGraph, LookupResult, TraversalDirection, impact, reach, search};
use sara_core::model::{Item, ItemSummary, ItemType, RelationshipType, TraceabilityLinks};
use sara_core::repository::{ScanResult, parse_repositories};
use sara_core::schema;
use sara_core::service::{
    EditOptions, EditResult, EditService, InitError, InitOptions, InitResult, InitService,
    TypeConfig, check_items, load_graph, read_body,
};
use tokio::sync::RwLock;
use tokio::task::spawn_blocking;

use crate::ServerOptions;
use crate::inputs::{field_inputs, relation_inputs};
use crate::params::{
    AnalyzeImpactParams, CreateItemParams, GetItemParams, IMPACT_LIMIT, MAX_IMPACT_ORIGINS,
    SEARCH_LIMIT, SearchItemsParams, TRACE_LIMIT, TraceDirection, TraceItemParams,
    UpdateItemParams, ValidateParams, parse_item_types, parse_one_item_type, type_list,
};
use crate::paths::resolve_item_file;
use crate::views::{ImpactView, ItemView, SearchResults, TraceView, ValidationView};

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

    /// Checks the whole graph: broken references, duplicate ids, cycles,
    /// invalid relationships, missing or invalid fields, orphan items and
    /// malformed ids, plus the files and repositories the scan skipped.
    /// Each issue names the items or files involved. Call it after creating
    /// or editing items.
    #[tool(
        name = "sara_validate",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn validate(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<Json<ValidationView>, String> {
        let strict = params.strict.unwrap_or(self.options.strict);

        self.read_repositories(move |repositories| {
            let ScanResult { items, warnings } = parse_repositories(repositories);
            let outcome = check_items(items, strict).map_err(|e| e.to_string())?;
            Ok(Json(ValidationView::new(outcome.report, warnings, strict)))
        })
        .await
    }

    /// Creates an item: writes a new Markdown file from the type's template,
    /// or adds a frontmatter to an existing Markdown file that has none. The
    /// id follows the model's id format unless given. Fields, relations and
    /// targets are checked first, and the file must be inside a repository,
    /// so nothing is written on a mistake. Required fields not given hold the
    /// model's placeholder and are listed in defaulted_fields: set them with
    /// sara_update_item. Write the body with your own file tools afterwards,
    /// then call sara_validate.
    #[tool(
        name = "sara_create_item",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn create_item(
        &self,
        Parameters(params): Parameters<CreateItemParams>,
    ) -> Result<Json<InitResult>, String> {
        let item_type = parse_one_item_type(&params.item_type)?;
        let fields = field_inputs(item_type, params.fields)?;
        let repositories = self.options.repositories.clone();
        let allowed: Vec<_> = item_type
            .declared_relations()
            .into_iter()
            .filter(|relation| !relation.is_downstream())
            .collect();

        self.write_graph(move |graph| {
            // Checked under the write lock, so no other request changes the
            // file system between the check and the write.
            let base = env::current_dir()
                .map_err(|e| format!("cannot read the working directory: {e}"))?;
            let file = resolve_item_file(&params.file, &repositories, &base)?;
            let relations = relation_inputs(graph, item_type, params.relations, &allowed)?;
            let mut config = TypeConfig::new(item_type);
            for (name, input) in fields {
                config = config.field(name, input);
            }
            for (relation, targets) in relations {
                config = config.relation(relation.as_str(), targets);
            }
            let options = InitOptions::new(file, config)
                .maybe_id(params.id)
                .maybe_name(params.name)
                .maybe_description(params.description);

            let initialized = InitService::new()
                .init(Some(graph), &options)
                .map_err(|error| match error {
                    InitError::FrontmatterExists(path) => format!(
                        "{} already holds an item; change it with sara_update_item.",
                        path.display()
                    ),
                    other => other.to_string(),
                })?;

            Ok(Json(initialized))
        })
        .await
    }

    /// Changes an item's frontmatter: name, description, field values and
    /// relation targets. Only the given entries are rewritten; everything
    /// else in the file stays as is, including links written there from the
    /// downstream side, keys unknown to the model, comments and the Markdown
    /// body. A relation list replaces the targets written in this item's
    /// file and an empty list removes the relation. Links written in the
    /// linked item's file are not changed and are listed in kept_links. Call
    /// sara_validate afterwards.
    #[tool(
        name = "sara_update_item",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn update_item(
        &self,
        Parameters(params): Parameters<UpdateItemParams>,
    ) -> Result<Json<EditResult>, String> {
        self.write_graph(move |graph| {
            let item = find_item(graph, &params.id)?;
            let fields = field_inputs(item.item_type, params.fields)?;
            let allowed: Vec<RelationshipType> = TraceabilityLinks::from_item(item)
                .iter()
                .map(|(relation, _)| relation)
                .collect();
            let relations = relation_inputs(graph, item.item_type, params.relations, &allowed)?;

            let mut options = EditOptions::new(item.id.as_str())
                .maybe_name(params.name)
                .maybe_description(params.description);
            for (name, input) in fields {
                options = options.with_field(name, input);
            }
            for (relation, targets) in relations {
                options = options.with_relation(relation, targets);
            }
            if !options.has_updates() {
                return Err(
                    "Nothing to change: give a name, a description, fields or relations."
                        .to_string(),
                );
            }

            EditService::new()
                .edit(graph, options)
                .map(Json)
                .map_err(|e| e.to_string())
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

    /// Runs `task` on the graph freshly loaded from the repositories, while
    /// no other request reads or writes the files.
    async fn write_graph<T: Send + 'static>(
        &self,
        task: impl FnOnce(&KnowledgeGraph) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        let _guard = self.lock.write().await;
        self.with_graph(task).await
    }

    /// Runs `task` on the configured repositories while other requests may
    /// read the files too.
    async fn read_repositories<T: Send + 'static>(
        &self,
        task: impl FnOnce(&[PathBuf]) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        let _guard = self.lock.read().await;
        self.on_repositories(task).await
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
             Before changing an item, list what it impacts with sara_analyze_impact. \
             Check the whole graph with sara_validate. \
             Create an item with sara_create_item, then write its body with your file tools. \
             Change an item's frontmatter with sara_update_item.",
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
