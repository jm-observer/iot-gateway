extern crate gateway_linker;

use anyhow::Result;
use async_channel::{Receiver, Sender};
use gateway_linker::*;
use log::{debug, error, warn};
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::Duration;
// use tokio::select;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    core().await?;
    Ok(())
}

struct Alloc(usize, u64);
/// position, start, end, time
struct Task(usize, usize, usize, u64);
struct Free(usize);

async fn core() -> Result<()> {
    let (task_sender, task_recver) = async_channel::bounded(1000);
    let (alloc_sender, alloc_recver) = async_channel::bounded(1000);
    let (free_sender, free_recver) = async_channel::bounded(1000);
    let mut memory = NodeManage::new();
    tokio::time::sleep(Duration::from_millis(500)).await;
    for _ in 0..10 {
        let free_sender_tmp = free_sender.clone();
        let task_recver_tmp = task_recver.clone();
        tokio::task::spawn(async move {
            deal_task(free_sender_tmp, task_recver_tmp).await;
        });
    }
    let alloc_sender_tmp = alloc_sender.clone();
    tokio::spawn(async move {
        product_task(alloc_sender_tmp).await;
        debug!("****************** product end **************");
        tokio::time::sleep(Duration::from_secs(100)).await;
    });
    let mut allo_tmp = 0;
    let mut time_tmp = 0;
    loop {
        while let Ok(Free(start)) = free_recver.try_recv() {
            debug!("start to free {}", start);
            if let Err(e) = memory.free(start) {
                error!("memory.free error: {:?}", e);
            } else {
                debug!("{} freed", start);
            }
        }
        if allo_tmp > 0 {
            match memory.alloc(allo_tmp) {
                Ok(start) => {
                    if let Err(e) = task_sender
                        .send(Task(start.0, start.1, allo_tmp, time_tmp))
                        .await
                    {
                        error!("task_sender.send error: {:?}", e);
                    }
                    allo_tmp = 0;
                }
                Err(e) => {
                    warn!("memory.alloc error: {:?}", e);
                }
            }
        } else {
            let mut i = 0;
            while let Ok(Alloc(len, time)) = alloc_recver.try_recv() {
                debug!("start to alloc {}", len);
                match memory.alloc(len) {
                    Ok(start) => {
                        if let Err(e) = task_sender.send(Task(start.0, start.1, len, time)).await {
                            error!("task_sender.send error: {:?}", e);
                        }
                    }
                    Err(e) => {
                        warn!("memory.alloc error: {:?}", e);
                        allo_tmp = len;
                        time_tmp = time;
                        break;
                    }
                }
                i += 1;
                if i > 20 {
                    break;
                }
            }
        }
        memory.check().unwrap();
    }
}

async fn product_task(sender: Sender<Alloc>) {
    let mut rng: StdRng = SeedableRng::from_entropy();
    // let mut rng = thread_rng();
    for _ in 0..1000 {
        let len = rng.gen_range(5..500);
        let time = rng.gen_range(20..100);
        if let Err(e) = sender.send(Alloc(len, time)).await {
            error!("{:?}", e);
        }
    }
}

async fn deal_task(sender: Sender<Free>, recver: Receiver<Task>) {
    loop {
        if let Ok(Task(position, start, _, time)) = recver.recv().await {
            tokio::time::sleep(Duration::from_millis(time)).await;
            debug!("{} dealed", start);
            if let Err(e) = sender.send(Free(position)).await {
                error!("{:?}", e);
            }
        } else {
            warn!("recver error");
        }
    }
}
