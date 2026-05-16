use crate::crawler::FileInfo;
use std::collections::HashMap;

pub type NodeId = usize;

#[derive(Debug)]
pub struct Node {
    pub name:     String,
    pub path:     String,
    pub size:     u64,
    pub is_dir:   bool,
    pub children: Vec<NodeId>,
    pub parent:   Option<NodeId>,
}

pub struct Arena {
    pub nodes: Vec<Node>,
}

impl Arena {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
    pub fn add(&mut self, node: Node) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }
    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }
    pub fn get_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id]
    }
}

fn get_or_create(
    arena:       &mut Arena,
    path_to_id:  &mut HashMap<String, NodeId>,
    path:        &str,
    root_path:   &str,
    root_id:     NodeId,
) -> NodeId {
    if let Some(&id) = path_to_id.get(path) {
        return id;
    }

    // Parent'ı da recursive olarak oluştur
    let parent_path = std::path::Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| root_path.to_string());

    let parent_id = if parent_path == root_path || parent_path.len() < root_path.len() {
        root_id
    } else {
        get_or_create(arena, path_to_id, &parent_path, root_path, root_id)
    };

    let name = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    let id = arena.add(Node {
        name,
        path: path.to_string(),
        size: 0,
        is_dir: true,
        children: Vec::new(),
        parent: Some(parent_id),
    });
    path_to_id.insert(path.to_string(), id);
    arena.get_mut(parent_id).children.push(id);
    id
}

pub fn build_tree(root_path: &str, files: Vec<FileInfo>) -> (Arena, NodeId) {
    let mut arena = Arena::new();
    let mut path_to_id: HashMap<String, NodeId> = HashMap::new();

    let root_id = arena.add(Node {
        name:     std::path::Path::new(root_path)
                      .file_name()
                      .map(|n| n.to_string_lossy().to_string())
                      .unwrap_or_else(|| root_path.to_string()),
        path:     root_path.to_string(),
        size:     0,
        is_dir:   true,
        children: Vec::new(),
        parent:   None,
    });
    path_to_id.insert(root_path.to_string(), root_id);

    for file in files {
        // Root'un kendisini atla
        if file.path == root_path {
            continue;
        }

        let parent_path = std::path::Path::new(&file.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| root_path.to_string());

        let parent_id = get_or_create(
            &mut arena,
            &mut path_to_id,
            &parent_path,
            root_path,
            root_id,
        );

        // Dosya zaten eklenmediyse ekle
        if path_to_id.contains_key(&file.path) {
            // Klasör olarak önceden oluşturulduysa size'ı güncelle
            if !file.is_dir {
                if let Some(&id) = path_to_id.get(&file.path) {
                    arena.get_mut(id).size = file.size;
                }
            }
            continue;
        }

        let node_id = arena.add(Node {
            name:     file.name,
            path:     file.path.clone(),
            size:     file.size,
            is_dir:   file.is_dir,
            children: Vec::new(),
            parent:   Some(parent_id),
        });
        path_to_id.insert(file.path, node_id);
        arena.get_mut(parent_id).children.push(node_id);
    }

    compute_sizes(&mut arena, root_id);
    (arena, root_id)
}

fn compute_sizes(arena: &mut Arena, node_id: NodeId) {
    let children: Vec<NodeId> = arena.get(node_id).children.clone();
    for child_id in children {
        compute_sizes(arena, child_id);
        let child_size = arena.get(child_id).size;
        arena.get_mut(node_id).size += child_size;
    }
}
