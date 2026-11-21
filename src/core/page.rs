//! Page tree structures.
#[derive(Debug, Clone, Default)]
pub struct FPageNode {
    /// Node type (Pages or Page).
    pub node_type: String,
    /// Parent node reference.
    pub parent: Option<u32>,
    /// Kids (children) references.
    pub kids: Vec<u32>,
    /// Page count (for Pages nodes).
    pub count: Option<u32>,
    /// Inherited media box.
    pub media_box: Option<[f32; 4]>,
    /// Inherited crop box.
    pub crop_box: Option<[f32; 4]>,
    /// Inherited resources.
    pub resources: Option<String>,
    /// Inherited rotation.
    pub rotation: Option<u32>,
}
/// The page tree structure.
#[derive(Debug, Clone, Default)]
pub struct FPageTree {
    /// Root page tree node reference.
    pub root: Option<u32>,
    /// All page tree nodes by object number.
    pub nodes: std::collections::HashMap<u32, FPageNode>,
}
impl FPageTree {
    /// Creates an empty page tree.
    pub fn new() -> Self {
        FPageTree::default()
    }
    /// Adds a node to the tree.
    pub fn add_node(&mut self, object_number: u32, node: FPageNode) {
        self.nodes.insert(object_number, node);
    }
    /// Gets a node by object number.
    pub fn get_node(&self, object_number: u32) -> Option<&FPageNode> {
        self.nodes.get(&object_number)
    }
    /// Gets all page object numbers in order.
    pub fn get_page_numbers(&self) -> Vec<u32> {
        let mut result = Vec::new();
        if let Some(root) = self.root {
            self.collect_pages(root, &mut result);
        }
        result
    }
    /// Recursively collects page numbers.
    fn collect_pages(&self, node_num: u32, result: &mut Vec<u32>) {
        if let Some(node) = self.nodes.get(&node_num) {
            if node.node_type == "Page" {
                result.push(node_num);
            } else {
                for &kid in &node.kids {
                    self.collect_pages(kid, result);
                }
            }
        }
    }
}
