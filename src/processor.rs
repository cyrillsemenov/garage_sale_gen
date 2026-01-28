use std::collections::BTreeMap;

use crate::error::ErrorCollector;
use crate::graph::Graph;
use crate::site_builder::PageId;
use crate::site_builder::SiteBuilder;
use crate::site_builder::models::NavNode;
use crate::site_builder::models::PageMeta;
use anyhow::Result;

pub struct Processor {
    graph: Graph<PageId>,
}

impl Processor {
    pub fn new(builder: &SiteBuilder, errors: &mut ErrorCollector) -> Result<Self> {
        let graph = Graph::build(builder, errors)?;
        Ok(Self { graph })
    }

    pub fn process(&mut self, builder: &mut SiteBuilder) -> Result<()> {
        let order = self.graph.order(builder);
        let mut children_map = BTreeMap::new();

        for page_id in order {
            self.process_page(builder, &page_id, &mut children_map)?;
        }

        Ok(())
    }

    pub fn graph(&self) -> &Graph<PageId> {
        &self.graph
    }

    fn process_page(
        &mut self,
        builder: &mut SiteBuilder,
        page_id: &PageId,
        children_map: &mut BTreeMap<PageId, Vec<PageMeta>>,
    ) -> Result<()> {
        // Apply parent attributes
        let parent_ids = self.apply_parent_attributes(builder, page_id)?;

        // Build page metadata
        let mut page_meta = self.build_page_meta(builder, page_id)?;

        // Attach children
        if let Some(children) = children_map.get(page_id) {
            page_meta.children.extend_from_slice(children);
        }

        // Convert markdown to HTML
        self.render_markdown(builder, page_id, &mut page_meta)?;

        // Register as child of parents
        for parent_id in parent_ids {
            children_map
                .entry(parent_id)
                .or_default()
                .push(page_meta.clone());
        }

        // Store processed metadata
        let page = builder
            .pages_mut()
            .get_mut(page_id)
            .ok_or_else(|| anyhow::anyhow!("Page not found: {:?}", page_id))?;
        page.page_meta = Some(page_meta);

        Ok(())
    }

    fn apply_parent_attributes(
        &self,
        builder: &mut SiteBuilder,
        page_id: &PageId,
    ) -> Result<Vec<PageId>> {
        let mut parent_ids = Vec::new();

        if let Some(parents) = self.graph.get_parents(page_id) {
            for parent_id in parents {
                let attrs = builder
                    .pages()
                    .get(parent_id)
                    .ok_or_else(|| anyhow::anyhow!("Parent page not found: {:?}", parent_id))?
                    .file_meta
                    .children_attrs
                    .clone();

                let page = builder
                    .pages_mut()
                    .get_mut(page_id)
                    .ok_or_else(|| anyhow::anyhow!("Page not found: {:?}", page_id))?;
                page.file_meta.append(attrs)?;

                parent_ids.push(parent_id.clone());
            }
        }

        Ok(parent_ids)
    }

    fn build_page_meta(&mut self, builder: &SiteBuilder, page_id: &PageId) -> Result<PageMeta> {
        let page = builder
            .pages()
            .get(page_id)
            .ok_or_else(|| anyhow::anyhow!("Page not found: {:?}", page_id))?;
        let mut meta: PageMeta = page.file_meta.clone().into();
        meta.stem = page.stem.clone();

        if let Some(segments) = self.graph.get_path_segments(page_id) {
            meta.path_segments = segments.clone();
        }

        let mut url = String::from("/");
        for seg in &meta.path_segments {
            url.push_str(seg);
            url.push('/');
        }

        if !(meta.stem == "index" && meta.path_segments.is_empty()) {
            url.push_str(&meta.stem);
            url.push_str(".html");
        } else {
            // Root index stays "/"
        }
        meta.url = url;
        meta.id = page_id.to_string();

        Ok(meta)
    }

    fn render_markdown(
        &self,
        builder: &SiteBuilder,
        page_id: &PageId,
        meta: &mut PageMeta,
    ) -> Result<()> {
        let page = builder
            .pages()
            .get(page_id)
            .ok_or_else(|| anyhow::anyhow!("Page not found: {:?}", page_id))?;

        meta.html_content = render_markdown_to_html(&page.content);
        Ok(())
    }

    pub fn build_navigation(&self, builder: &SiteBuilder) -> Result<Vec<NavNode>> {
        let mut roots = Vec::new();

        for page_id in builder.pages().keys() {
            if let Some(parents) = self.graph.get_parents(page_id) {
                if parents.is_empty() {
                    roots.push(page_id.clone());
                }
            } else {
                roots.push(page_id.clone());
            }
        }

        roots.sort();

        let mut nav_roots = Vec::new();

        for root_id in roots {
            if let Some(nav) = self.build_nav_node(builder, &root_id)? {
                nav_roots.push(nav);
            }
        }

        Ok(nav_roots)
    }

    pub fn compute_breadcrumbs(
        &self,
        builder: &SiteBuilder,
    ) -> Result<BTreeMap<PageId, Vec<NavNode>>> {
        use crate::site_builder::models::NavNode;
        let mut crumbs_map = BTreeMap::new();

        for page_id in builder.pages().keys() {
            let mut crumbs = Vec::new();
            let mut current = page_id.clone();
            let mut seen = std::collections::HashSet::new();

            // Trace up to root
            while let Some(parents) = self.graph.get_parents(&current) {
                if parents.is_empty() {
                    break;
                }
                // Take the first parent (arbitrary for now if multiple)
                let parent = parents.iter().next().unwrap();

                if !seen.insert(parent.clone()) {
                    break; // Cycle detected
                }

                // Get parent metadata
                if let Some(page) = builder.pages().get(parent) {
                    if let Some(meta) = &page.page_meta {
                        if meta.publish {
                            crumbs.push(NavNode {
                                title: meta.title.clone(),
                                url: meta.url.clone(),
                                children: Vec::new(),
                                order: None, // Order not relevant for breadcrumb items
                            });
                        }
                    }
                }
                current = parent.clone();
            }
            crumbs.reverse();
            crumbs_map.insert(page_id.clone(), crumbs);
        }
        Ok(crumbs_map)
    }

    fn build_nav_node(&self, builder: &SiteBuilder, page_id: &PageId) -> Result<Option<NavNode>> {
        use crate::site_builder::models::NavNode;

        let page = builder
            .pages()
            .get(page_id)
            .ok_or_else(|| anyhow::anyhow!("Page not found: {:?}", page_id))?;

        // Use page_meta if available (it has the URL computed), otherwise fallback to something else?
        // At this stage, page_meta MUST be populated because we run this after process()
        let meta = match &page.page_meta {
            Some(m) => m,
            None => return Ok(None), // Should not happen if processed
        };

        if !meta.publish {
            return Ok(None);
        }

        let order = meta
            .extra
            .get("order")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32);

        let mut node = NavNode {
            title: meta.title.clone(),
            url: meta.url.clone(),
            children: Vec::new(),
            order,
        };

        for child_path_str in &page.file_meta.children {
            let child_id = PageId::from(child_path_str.as_str());

            if builder.pages().contains_key(&child_id) {
                if let Some(child_node) = self.build_nav_node(builder, &child_id)? {
                    node.children.push(child_node);
                }
            } else {
                let possible_index_id = PageId::from(format!("{}/index", child_path_str));
                if builder.pages().contains_key(&possible_index_id) {
                    if let Some(child_node) = self.build_nav_node(builder, &possible_index_id)? {
                        node.children.push(child_node);
                    }
                }
            }
        }

        Ok(Some(node))
    }
}

fn render_markdown_to_html(markdown: &str) -> String {
    let parser = pulldown_cmark::Parser::new_ext(markdown, pulldown_cmark::Options::all());
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

impl Processor {
    pub fn rebuild(&mut self, builder: &mut SiteBuilder, page_id: &PageId) -> Result<Vec<PageId>> {
        // let source = builder.pages.get(page_id).unwrap();
        let affected = self.graph.rebuild(builder, page_id)?;

        let mut children_map = BTreeMap::new();
        for id in &affected {
            self.process_page(builder, id, &mut children_map)?;
        }

        Ok(affected)
    }
}
