use anyhow::Result;
use log::{debug, trace};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

/// Trait for nodes in the dependency graph
pub(crate) trait GraphNode {
    /// Returns the list of child collection roots for this node
    fn child_roots(&self) -> &[String];

    /// Returns true if this is an index node (aggregates children from parent dir)
    fn is_index(&self) -> bool;

    /// Returns the parent directory path
    fn parent_dir(&self) -> &Path;
}

/// Dependency graph for pages
#[derive(Debug)]
pub(crate) struct Graph<Id> {
    /// Maps each page to its set of parent pages
    parents_of: BTreeMap<Id, BTreeSet<Id>>,
    /// Maps each page to its set of child pages (for efficient traversal)
    children_of: BTreeMap<Id, BTreeSet<Id>>,
    /// Tracks incoming edge count for each node (used for topological sort)
    incoming_edges: BTreeMap<Id, usize>,
    /// Path segments for each page (used in URL construction)
    pub(crate) path_segments: BTreeMap<Id, Vec<String>>,
    /// Root directories that contain collections of pages
    pub(crate) collection_roots: BTreeSet<PathBuf>,
}

impl<Id: Ord + Clone + std::fmt::Debug + From<PathBuf>> Graph<Id> {
    /// Creates a new empty graph
    pub(crate) fn new() -> Self {
        Self {
            parents_of: BTreeMap::new(),
            children_of: BTreeMap::new(),
            incoming_edges: BTreeMap::new(),
            path_segments: BTreeMap::new(),
            collection_roots: BTreeSet::new(),
        }
    }

    /// Ensures a node exists in the graph with initialized data structures
    fn touch_node(&mut self, id: &Id) {
        self.incoming_edges.entry(id.clone()).or_insert(0);
        self.parents_of.entry(id.clone()).or_default();
        self.children_of.entry(id.clone()).or_default();
        self.path_segments
            .entry(id.clone())
            .or_insert_with(Vec::new);
    }

    /// Removes all edges where the given node is a parent
    fn detach_parent(&mut self, id: &Id) {
        trace!("Detaching parent {:?}", id);
        if let Some(children) = self.children_of.remove(id) {
            for child in children {
                if let Some(parents) = self.parents_of.get_mut(&child) {
                    parents.remove(id);
                }
            }
            // Since we removed all children for this parent, its incoming edge count (which counts children) becomes 0
            if let Some(i) = self.incoming_edges.get_mut(id) {
                *i = 0;
            }
        }
        // Re-insert empty set since this node still exists
        self.children_of.insert(id.clone(), BTreeSet::new());
    }

    /// Attaches children to a parent node
    fn attach_children<I>(&mut self, id: &Id, children: I)
    where
        I: IntoIterator<Item = (Id, Vec<String>)>,
    {
        for (cid, segs) in children {
            trace!("Attaching child {:?} to parent {:?}", cid, id);
            self.path_segments.insert(cid.clone(), segs);

            // Update children_of for parent
            self.children_of
                .entry(id.clone())
                .or_default()
                .insert(cid.clone());

            // Update parents_of for child
            let parents = self.parents_of.entry(cid.clone()).or_default();
            if parents.insert(id.clone()) {
                *self.incoming_edges.entry(id.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Computes children for a given node
    /// Supports both directory paths and specific file references
    fn compute_children<'a, R, N>(
        &self,
        registry: &'a R,
        source: &'a N,
    ) -> impl Iterator<Item = (Id, Vec<String>)> + 'a
    where
        R: UnderRoot<Id> + Pages<Id, N>,
        N: GraphNode,
        Id: 'a,
    {
        let mut results = Vec::new();

        if !source.child_roots().is_empty() {
            for ch in source.child_roots() {
                let path = PathBuf::from(ch);

                // Check if it's a known collection root (directory)
                if self.collection_roots.contains(&path) {
                    // Directory - use existing under_root logic
                    results.extend(registry.under_root(&path));
                } else {
                    // Assume it's a file reference - try to find it
                    let page_id: Id = path.clone().into();
                    if registry.get_pages().contains_key(&page_id) {
                        // File exists, add with empty path segments
                        results.push((page_id, Vec::new()));
                    }
                    // If file doesn't exist, silently skip (warning handled in build)
                }
            }
        } else if source.is_index() {
            let root = source.parent_dir().to_path_buf();
            results.extend(registry.under_root(&root));
        }
        results.into_iter()
    }

    /// Builds the complete dependency graph from a registry
    /// Collects warnings for missing child references
    pub(crate) fn build<R, N>(
        registry: &R,
        errors: &mut crate::error::ErrorCollector,
    ) -> Result<Self>
    where
        R: Pages<Id, N> + UnderRoot<Id>,
        N: GraphNode,
    {
        debug!("Building dependency graph...");
        let mut g = Self::new();

        for id in registry.page_ids() {
            g.touch_node(id);
        }

        // Pre-calculate strict directory prefixes to optimize checks
        // We collect all unique parent directories from all pages
        let mut known_dirs = BTreeSet::new();
        for page in registry.get_pages().values() {
            let p = page.parent_dir();
            // Store all ancestors of this parent dir effectively?
            // Actually, we just need to know if a path in `child_roots` is a prefix of any actual content locations.
            // If `child_roots` says "posts", and we have "posts/my-post.md", then "posts" is a directory.
            known_dirs.insert(p.to_path_buf());
        }

        for (pid, node) in registry.get_pages() {
            if !node.child_roots().is_empty() {
                for ch in node.child_roots() {
                    let path = PathBuf::from(ch);

                    // Optimized check: path is a directory if it IS one of the known parent dirs
                    // or if it is a prefix of one of them.
                    // Since known_dirs contains full parent paths, we can check if `path` is contained
                    // or if any known_dir starts with `path`.
                    // Checking `any` is still linear in number of dirs, but number of dirs << number of pages.
                    let is_directory = known_dirs.iter().any(|d| d.starts_with(&path));

                    if is_directory {
                        // It's a directory - add to collection roots
                        g.collection_roots.insert(path);
                    } else {
                        // It's a file reference - validate it exists
                        let page_id: Id = path.clone().into();
                        if !registry.get_pages().contains_key(&page_id) {
                            // Missing file reference - collect warning
                            errors.add_warning(crate::error::ParserError::missing_child_reference(
                                pid.clone(),
                                ch.clone(),
                            ));
                        }
                    }
                }
            } else if node.is_index() {
                g.collection_roots.insert(node.parent_dir().to_path_buf());
            }

            let children = g.compute_children(registry, node);
            g.attach_children(pid, children);
        }

        debug!(
            "Graph built: {} nodes, {} collection roots",
            g.incoming_edges.len(),
            g.collection_roots.len()
        );

        Ok(g)
    }

    /// Rebuilds the graph for a specific node and returns affected nodes in topological order
    pub(crate) fn rebuild<R, N>(&mut self, registry: &R, id: &Id) -> Result<Vec<Id>>
    where
        R: Pages<Id, N> + UnderRoot<Id>,
        N: GraphNode,
    {
        debug!("Rebuilding graph for node {:?}", id);
        self.touch_node(id);
        self.detach_parent(id);

        let source = registry.get_pages().get(id).ok_or_else(|| {
            anyhow::anyhow!("Node with ID {:?} not found in registry during rebuild", id)
        })?;

        if !source.child_roots().is_empty() {
            for ch in source.child_roots() {
                self.collection_roots.insert(PathBuf::from(ch));
            }
        } else if source.is_index() {
            self.collection_roots
                .insert(source.parent_dir().to_path_buf());
        }
        let children = self.compute_children(registry, source);
        self.attach_children(id, children);

        // Find all affected nodes (the changed node and all its ancestors)
        let mut affected: BTreeSet<Id> = BTreeSet::new();
        let mut stack = vec![id.clone()];
        affected.insert(id.clone());
        while let Some(n) = stack.pop() {
            if let Some(pars) = self.parents_of.get(&n) {
                for p in pars {
                    if affected.insert(p.clone()) {
                        stack.push(p.clone());
                    }
                }
            }
        }

        // Compute local in-degrees within the affected subgraph
        let mut indeg_local: BTreeMap<Id, usize> = BTreeMap::new();
        for node in &affected {
            let mut deg = 0;
            for (child, parents) in self.parents_of.iter() {
                if affected.contains(child) && parents.contains(node) {
                    deg += 1;
                }
            }
            indeg_local.insert(node.clone(), deg);
        }

        // Topological sort of affected subgraph
        let mut q: VecDeque<Id> = indeg_local
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(n, _)| n.clone())
            .collect();

        let mut order = Vec::with_capacity(affected.len());
        while let Some(u) = q.pop_front() {
            order.push(u.clone());
            if let Some(parents) = self.parents_of.get(&u) {
                for p in parents {
                    if !affected.contains(p) {
                        continue;
                    }
                    if let Some(d) = indeg_local.get_mut(p) {
                        *d -= 1;
                        if *d == 0 {
                            q.push_back(p.clone());
                        }
                    }
                }
            }
        }

        if order.len() != affected.len() {
            anyhow::bail!("cycle detected in affected subgraph starting at {:?}", id);
        }

        debug!("Rebuild complete, re-ordered {} nodes", order.len());
        Ok(order)
    }

    /// Returns all nodes in topological order (children before parents)
    pub(crate) fn order<R, N>(&mut self, registry: &R) -> Vec<Id>
    where
        R: Pages<Id, N>,
    {
        debug!("Topological sort started");
        // Work on a copy of incoming_edges to avoid destroying graph state
        let mut incoming = self.incoming_edges.clone();

        let mut q: VecDeque<Id> = registry
            .get_pages()
            .keys()
            .filter(|pid| incoming.get(*pid).copied().unwrap_or(0) == 0)
            .cloned()
            .collect();

        let mut out = Vec::with_capacity(registry.get_pages().len());

        while let Some(u) = q.pop_front() {
            out.push(u.clone());

            // Iterate parents and decrement their dependency count
            // Note: edges are Child -> Parent (logic wise here)
            if let Some(parents) = self.parents_of.get(&u) {
                for p in parents {
                    if let Some(e) = incoming.get_mut(p) {
                        *e = e.saturating_sub(1);
                        if *e == 0 {
                            q.push_back(p.clone());
                        }
                    }
                }
            }
        }

        debug!("Topological sort complete, sorted {} nodes", out.len());
        out
    }

    /// Gets the parent nodes for a given node ID
    pub(crate) fn get_parents(&self, id: &Id) -> Option<&BTreeSet<Id>> {
        self.parents_of.get(id)
    }

    /// Gets the path segments for a given node ID
    pub(crate) fn get_path_segments(&self, id: &Id) -> Option<&Vec<String>> {
        self.path_segments.get(id)
    }
}

/// Trait for types that can query nodes under a root directory
pub(crate) trait UnderRoot<Id> {
    fn under_root(&self, root: &Path) -> Vec<(Id, Vec<String>)>;
}

/// Trait for types that provide access to nodes
pub(crate) trait Pages<Id: Clone, Node> {
    fn get_pages(&self) -> &BTreeMap<Id, Node>;

    /// Returns an iterator over page IDs (zero-cost, no cloning)
    fn page_ids<'a>(&'a self) -> impl Iterator<Item = &'a Id> + 'a
    where
        Id: 'a,
        Node: 'a,
    {
        self.get_pages().keys()
    }
}
