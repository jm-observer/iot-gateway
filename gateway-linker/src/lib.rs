pub mod lib2;

use anyhow::{bail, Result};

const MAX: usize = 1000;
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
        for i in 0..MAX {
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
        if self.start != 0 {
            let position = self.data[self.start];
            // 重置为0
            self.data[self.start] = 0;
            if self.start != self.end {
                self.start += 1;
                if self.start >= MAX {
                    self.start = FIFO_MIN_INDEX;
                }
            } else {
                self.start = 0;
                self.end = 0;
            }
            return Ok(position);
        } else {
            bail!("无空闲位置")
        }
    }
    pub fn push(&mut self, position: usize) -> Result<()> {
        if self.end == 0 {
            self.start = 1;
            self.end = 1;
            self.data[1] = position;
            return Ok(());
        } else {
            let mut tmp_end = self.end + 1;
            if tmp_end >= MAX {
                tmp_end = FIFO_MIN_INDEX;
            }
            if tmp_end == self.start {
                bail!("程序出错");
            } else {
                self.data[tmp_end] = position;
                self.end = tmp_end;
            }
            Ok(())
        }
    }
    #[allow(dead_code)]
    pub fn check(&self) -> Result<()> {
        if self.start < self.end {
            for index in 1..self.start {
                if self.data[index] != 0 {
                    bail!("先入先出空闲异常：位置{}值为{}", index, self.data[index]);
                }
            }
            for index in self.start..=self.end {
                if self.data[index] == 0 {
                    bail!("先入先出占用位置异常：位置{}值为0", index);
                }
            }
            let index_start = self.end + 1;
            if index_start < MAX {
                for index in index_start..MAX {
                    if self.data[index] != 0 {
                        bail!("先入先出空闲异常：位置{}值为{}", index, self.data[index]);
                    }
                }
            }
        } else if self.start != 0 {
            for index in 1..self.end {
                if self.data[index] != 0 {
                    bail!("先入先出空闲异常：位置{}值为{}", index, self.data[index]);
                }
            }
            for index in self.end..=self.start {
                if self.data[index] == 0 {
                    bail!("先入先出占用位置异常：位置{}值为0", index);
                }
            }
            let index_start = self.start + 1;
            if index_start < MAX {
                for index in index_start..MAX {
                    if self.data[index] != 0 {
                        bail!("先入先出空闲异常：位置{}值为{}", index, self.data[index]);
                    }
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone)]
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
    const DEFAULT: Self = Node {
        front: 0,
        behind: 0,
        position: 0,
        alloc_start: 0,
        alloc_end: 0,
    };
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
    // start: usize,
    /// 寻址节点的位置
    index: usize,
    /// 内存大小
    memory_size: usize,
    fifo: Fifo,
}
impl NodeManage {
    pub fn new() -> Self {
        let mut nodes = [Node::DEFAULT; MAX];
        nodes[0].alloc_end = 1;
        Self {
            nodes,
            // start: 0,
            index: 0,
            memory_size: MAX * 10,
            fifo: Fifo::new(),
        }
    }
    pub fn free(&mut self, position: usize) -> Result<()> {
        if position == 0 {
            bail!("0节点不能释放");
        }
        if self.nodes[position].position != position {
            bail!("节点{}异常：节点值不对", position);
        }
        let behind_index = self.nodes[position].behind;
        let front_index = self.nodes[position].behind;
        // assert_ne!(0, node.front);
        self.nodes[front_index].behind = behind_index;
        if behind_index != 0 {
            self.nodes[behind_index].front = front_index;
        }
        if position == self.index {
            self.index = front_index;
        }
        self.fifo.push(position)?;
        Ok(())
    }
    /// 返回 位置、内存起始位置
    pub fn alloc(&mut self, len: usize) -> Result<(usize, usize)> {
        let mut start = self.nodes[self.index].alloc_end;
        let mut end = start + len;
        if end > self.memory_size {
            // 从头开始
            self.index = 0;
            if self.nodes[self.index].behind == 0 {
                bail!(
                    "异常情况：只会出现分配内存（{}）大于max（{}）的情况",
                    len,
                    self.memory_size
                );
            }
            loop {
                let node = self.nodes[self.index].behind;
                if node == 0 {
                    bail!("已到达末尾节点，依旧没有找到合适的内存空间");
                }
                start = self.nodes[self.index].alloc_end;
                end = start + len;
                if self.nodes[node].alloc_start > end {
                    let position = self.init_middle_node(node, start, end)?;
                    // 该空间足够
                    return Ok((position, start));
                } else {
                    self.index = node;
                }
            }
            // bail!("内存空间不足：{:?}, node.len={:?}", len, self.nodes.len());
        } else {
            let position = self.init_end_node(start, end)?;
            self.nodes[self.index].update_behind(position);
            self.index = position;
            return Ok((position, start));
        }
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
    pub fn check(&self) -> Result<()> {
        // 节点数
        let mut node_num = 0usize;
        let mut index = 0;
        let mut is_end = false;
        // 检查链表的节点数
        loop {
            let behind = self.nodes[index].behind;
            if index == self.index {
                is_end = true;
            }
            if behind == 0 {
                break;
            } else {
                node_num += 1;
                if self.nodes[behind].front != index {
                    bail!(
                        "节点{}的前置节点{}有误：正确值{}",
                        behind,
                        self.nodes[behind].front,
                        index
                    );
                }
                if self.nodes[behind].position != behind {
                    bail!(
                        "节点{}的位置值{}有误：正确值{}",
                        behind,
                        self.nodes[behind].position,
                        behind
                    );
                }
                index = behind;
            }
        }
        if !is_end {
            bail!("链表找不到寻址节点");
        }
        if self.index == 0 {
            assert_eq!(node_num, 0);
        }
        println!("the node's num is {}", node_num);
        Ok(())
    }
}

#[test]
fn test_other() {
    use std::mem;
    struct A(usize, usize, usize, usize, usize);
    println!("{}", mem::size_of::<Node>());
    println!("{}", mem::size_of::<A>());
}

#[test]
fn test_nm() {
    assert_eq!(10, MAX);
    let mut nm = NodeManage::new();
    // 普通alloc
    let position = nm.alloc(99);
    assert!(position.is_ok());
    assert!(nm.check().is_ok());
    // 普通free
    assert!(nm.free(position.unwrap().0).is_ok());
    assert!(nm.check().is_ok());
    // 异常alloc
    assert!(nm.alloc(100).is_err());
    assert!(nm.free(0).is_err());
    assert!(nm.free(2).is_err());
    assert!(nm.check().is_ok());
}

#[test]
fn test_fifo() {
    use rand::prelude::SliceRandom;
    assert_eq!(6, MAX);
    let mut fifo = Fifo::new();
    let mut tr = rand::thread_rng();
    let mut num: [usize; 5] = [5, 1, 2, 3, 4];
    num.shuffle(&mut tr);
    // 普通pop
    for _ in 1..MAX {
        assert!(fifo.pop().is_ok());
    }
    // 异常pop
    assert!(fifo.pop().is_err());
    assert!(fifo.check().is_ok());
    // 普通push
    for i in 0..(MAX - 1) {
        assert!(fifo.push(num[i]).is_ok());
    }
    // 异常push
    assert!(fifo.push(num[1]).is_err());
    assert!(fifo.check().is_ok());
    // 普通check
    assert!(fifo.pop().is_ok());
    assert!(fifo.pop().is_ok());
    assert!(fifo.check().is_ok());
}
