use anyhow::{anyhow, bail, Result};
use log::debug;
use std::collections::HashMap;
use std::ops::Deref;
use std::ptr::NonNull;

pub struct NodeManage {
    max: usize,
    // 占位
    start: Node,
    end: Node,
    nodes: HashMap<usize, NodeInner>,
}

impl NodeManage {
    pub fn new(max: usize) -> Result<Self> {
        let nodes = HashMap::new();
        let node = Node(NonNull::dangling());
        let mut nm = Self {
            max,
            start: node.clone(),
            end: node,
            nodes,
        };
        nm.add_start_node()?;
        Ok(nm)
    }
    fn add_middle_node(&mut self, start: usize, end: usize, behind: Node) -> Result<()> {
        let inner = NodeInner::new(start, end, Some(self.end.clone()), Some(behind));
        let node = self.init_node(start, inner)?;
        self.end.set_behind(Some(node));
        behind.set_front(Some(node));
        self.end = node;
        Ok(())
    }
    fn add_end_node(&mut self, start: usize, end: usize) -> Result<()> {
        if start == 11 {}
        let inner = NodeInner::new(start, end, Some(self.end.clone()), None);
        let node = self.init_node(start, inner)?;
        self.end.set_behind(Some(node));
        self.end.print();
        self.end = node;
        Ok(())
    }
    fn add_start_node(&mut self) -> Result<()> {
        let start = 0usize;
        let inner = NodeInner::new(start, 1, None, None);
        let mut node = self.init_node(start, inner)?;
        node.set_front(Some(node));
        self.start = node.clone();
        self.end = node;
        Ok(())
    }
    fn init_node(&mut self, start: usize, inner: NodeInner) -> Result<Node> {
        if self.nodes.insert(start, inner).is_some() {}
        self.nodes
            .get(&start)
            .ok_or(anyhow!("unreach error!"))
            .map(|x| Node(NonNull::new(x as *const NodeInner as *mut NodeInner).unwrap()))
    }
    pub fn free(&mut self, index: usize) -> Result<()> {
        let mut node = self
            .nodes
            .remove(&index)
            .ok_or(anyhow!("释放异常：未找到节点({})", index))?;
        // node.print();
        let behind = node.behind;
        let front = node
            .front
            .take()
            .ok_or(anyhow!("释放节点异常：该节点（{}）无前置节点", index))?;
        front.set_behind(behind);
        let end_index = self.end.start();
        // 更新end、behind节点
        if let Some(behind) = behind {
            behind.set_front(Some(front));
            if end_index == index {
                self.end = behind;
            }
        } else if end_index == index {
            self.end = front
        }
        Ok(())
    }
    pub fn alloc(&mut self, len: usize) -> Result<usize> {
        let mut start = self.end.end();
        let mut end = start + len;
        if end >= self.max {
            // 从头开始
            self.end = self.start;
            if self.end.behind().is_none() {
                bail!(
                    "异常情况：只会出现分配内存（{}）大于max（{}）的情况",
                    len,
                    self.max
                );
            }
            while let Some(node) = self.end.behind() {
                start = self.end.end();
                end = start + len;
                if node.start() > end {
                    self.add_middle_node(start, end, node)?;
                    // 该空间足够
                    return Ok(start);
                } else {
                    self.end = node;
                }
            }
            bail!("内存空间不足：{:?}, node.len={:?}", len, self.nodes.len());
        } else {
            self.add_end_node(start, end)?;
            return Ok(start);
        }
    }

    pub fn print(&self) {
        debug!("print:");
        let mut tmp_node = self.start;
        tmp_node.print();
        while let Some(next) = tmp_node.behind() {
            next.print();
            tmp_node = next;
        }
        debug!("MAP:");
        self.nodes.iter().for_each(|(index, val)| {
            debug!("index: {}", index);
            val.print();
        });
    }
}

#[derive(Clone, Copy)]
/// 实际为NodeInner的非空指针，包装是便于实现特征
struct Node(NonNull<NodeInner>);
impl Deref for Node {
    type Target = NonNull<NodeInner>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Node {
    fn print(&self) {
        unsafe {
            self.as_ref().print();
        }
    }
    fn behind(&self) -> Option<Node> {
        unsafe { (*self.as_ptr()).behind.clone() }
    }
    fn front(&self) -> Option<Node> {
        unsafe { (*self.as_ptr()).front.clone() }
    }
    fn end(&self) -> usize {
        unsafe { (*self.as_ptr()).end }
    }
    fn start(&self) -> usize {
        unsafe { (*self.as_ptr()).start }
    }

    fn set_front(&self, front: Option<Node>) -> Option<Node> {
        unsafe {
            let res = (*self.as_ptr()).front.take();
            (*self.as_ptr()).front = front;
            res
        }
    }
    fn set_behind(&self, behind: Option<Node>) -> Option<Node> {
        unsafe {
            let res = (*self.as_ptr()).behind.take();
            (*self.as_ptr()).behind = behind;
            res
        }
    }
}

struct NodeInner {
    front: Option<Node>,
    behind: Option<Node>,
    start: usize,
    end: usize,
}
impl NodeInner {
    pub fn new(start: usize, end: usize, front: Option<Node>, behind: Option<Node>) -> NodeInner {
        Self {
            start,
            end,
            front,
            behind,
        }
    }
    fn print(&self) {
        if let Some(front) = self.front {
            if let Some(behind) = self.behind {
                debug!(
                    "start:{}, end: {}, front: {:?}, self: {:?}, behind: {:?}",
                    self.start,
                    self.end,
                    front.as_ptr(),
                    self as *const NodeInner,
                    behind.as_ptr()
                );
            } else {
                debug!(
                    "start:{}, end: {}, front: {:?}, self: {:?}",
                    self.start,
                    self.end,
                    front.as_ptr(),
                    self as *const NodeInner,
                );
            }
        } else {
            panic!("!!!!!!!");
        }
    }
}
/// 只是为了便于观察
impl Drop for NodeInner {
    fn drop(&mut self) {
        println!("drop node：{:?}", self.start);
    }
}
#[test]
fn test_node_inner() {
    unsafe {
        let a = NodeInner::new(1, 2, None, None);
        let b = NodeInner::new(4, 5, None, None);
        let mut map = HashMap::new();

        map.insert(a.start, a);
        let a_ptr = map.get_mut(&1).unwrap();
        let a_node = Node(NonNull::new_unchecked(a_ptr as *mut NodeInner));
        map.insert(b.start, b);
        let b_ptr = map.get_mut(&4).unwrap();
        let b_node = Node(NonNull::new_unchecked(b_ptr as *mut NodeInner));

        let c = NodeInner::new(2, 4, Some(a_node), Some(b_node));
    }
}

#[test]
fn test_node_manage() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let mut nm = NodeManage::new(1000).unwrap();
    // nm.print();
    assert_eq!(1, nm.alloc(5)?);
    assert_eq!(6, nm.alloc(5)?);
    assert_eq!(11, nm.alloc(5)?);
    // assert_eq!(16, nm.alloc(5)?);
    nm.print();
    // debug!("*****************");
    // assert!(nm.free(1).is_ok());
    // nm.print();

    // assert_eq!(true, nm.alloc(1000).is_err());
    // // println!("start to free");
    // nm.print();
    // assert!(nm.free(6).is_ok());
    // nm.print();
    // assert!(nm.free(16).is_ok());
    // nm.print();

    // assert!(nm.free(11).is_ok());
    // nm.print();
    // assert!(nm.free(70).is_err());
    // nm.print();
    Ok(())
}
