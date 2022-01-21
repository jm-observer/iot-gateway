pub mod lib2;

use anyhow::{bail, Result};

const MAX: usize = 10000;
// 0作为特殊值，为空的意思。保证0节点不会被释放
const FIFO_MIN_INDEX: usize = 1;
///先入先出
struct Fifo {
    data: [usize; MAX],
    start: usize,
    end: usize,
}

impl Fifo {
    fn new() -> Self {
        let mut data = [0usize; MAX];
        let mut tmp = 0usize;
        for i in 0..=MAX {
            data[i] = i;
        }
        println!("{} - {}", data[0], data[MAX - 1]);
        Self {
            data,
            start: 1,
            end: MAX - 1,
        }
    }
    pub fn pop(&mut self) -> Result<usize> {
        if self.start != self.end {
            let position = self.start;
            self.start += 1;
            if self.start >= MAX {
                self.start = FIFO_MIN_INDEX;
            }
            return Ok(position);
        } else {
            bail!("Fifo无空闲位置");
        }
    }
    pub fn push(&mut self, position: usize) -> Result<()> {
        self.end += 1;
        if self.end >= MAX {
            self.end = FIFO_MIN_INDEX;
        }
        if self.end == self.start {
            bail!("程序出错");
        } else {
            self.end = position;
        }
        Ok(())
    }
}
#[derive(Copy, Clone)]
struct Node {
    /// 前一节点在nodes数组中的index
    front: usize,
    /// 后一节点在nodes数组中的index
    behind: usize,
    /// 在nodes数组中的index
    position: usize,
    /// 在内存中起始位置
    alloc_start: usize,
    /// 在内存中结束位置（不包括）
    alloc_end: usize,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            front: 0,
            behind: 0,
            position: 0,
            alloc_start: 0,
            alloc_end: 0,
        }
    }
}
impl Node {
    fn init_end_node(
        &mut self,
        front: usize,
        position: usize,
        alloc_start: usize,
        alloc_end: usize,
    ) {
        self.init_node(front, 0, position, alloc_start, alloc_end);
    }
    fn init_node(
        &mut self,
        front: usize,
        behind: usize,
        position: usize,
        alloc_start: usize,
        alloc_end: usize,
    ) {
        self.front = front;
        self.behind = behind;
        self.position = position;
        self.alloc_start = alloc_start;
        self.alloc_end = alloc_end;
    }
    fn update_behind(&mut self, behind: usize) {
        self.behind = behind;
    }
}

pub struct NodeManage {
    nodes: [Node; MAX],
    /// 起始节点的位置
    start: usize,
    /// 寻址节点的位置
    index: usize,
    /// 内存大小
    memory_size: usize,
    fifo: Fifo,
}
impl NodeManage {
    pub fn new() -> Self {
        let mut nodes = [Node::default(); MAX];
        nodes[0].alloc_end = 1;
        Self {
            nodes: [Node::default(); MAX],
            start: 0,
            index: 0,
            memory_size: MAX * 10,
            fifo: Fifo::new(),
        }
    }
    pub fn free(&mut self, position: usize) -> Result<()> {
        let node = self.nodes[position];
        assert_ne!(0, node.front);
        self.nodes[node.front].behind = node.behind;
        if node.behind != 0 {
            self.nodes[node.behind].front = node.front;
        }
        if position == self.index {
            self.index = node.front;
        }
        Ok(())
    }
    pub fn alloc(&mut self, len: usize) -> Result<usize> {
        let mut start = self.nodes[self.index].alloc_end;
        let mut end = start + len;
        if end >= self.memory_size {
            // 从头开始
            self.index = self.start;
            if self.nodes[self.index].behind == 0 {
                bail!(
                    "异常情况：只会出现分配内存（{}）大于max（{}）的情况",
                    len,
                    self.memory_size
                );
            }
            while let node = self.nodes[self.index].behind {
                if node == 0 {
                    bail!("??????????");
                }
                start = self.nodes[self.index].alloc_end;
                end = start + len;
                if self.nodes[node].alloc_start > end {
                    let position = self.init_middle_node(node, start, end)?;
                    // 该空间足够
                    return Ok(start);
                } else {
                    self.index = node;
                }
            }
            bail!("内存空间不足：{:?}, node.len={:?}", len, self.nodes.len());
        } else {
            let position = self.init_end_node(start, end)?;
            self.nodes[self.index].update_behind(position);
            self.index = position;
            return Ok(start);
        }
        Ok(0)
    }
    fn init_end_node(&mut self, start: usize, end: usize) -> Result<usize> {
        let position = self.fifo.pop()?;
        self.nodes[position].init_end_node(self.index, position, start, end);
        Ok(position)
    }
    fn init_middle_node(&mut self, behind: usize, start: usize, end: usize) -> Result<usize> {
        let position = self.fifo.pop()?;
        self.nodes[position].init_node(self.index, behind, position, start, end);
        self.nodes[self.index].behind = position;
        self.nodes[behind].front = position;
        self.index = position;
        Ok(position)
    }
}
