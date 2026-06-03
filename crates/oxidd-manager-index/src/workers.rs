use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// Worker thread pool
pub struct Workers {
    #[cfg(not(feature = "lace"))]
    pub(crate) pool: rayon::ThreadPool,
    split_depth: AtomicU32,
    #[cfg(feature = "lace")]
    num_threads: usize,
}

impl Workers {
    #[cfg(not(feature = "lace"))]
    pub(crate) fn new(threads: u32) -> Self {
        let stack_size = std::env::var("OXIDD_STACK_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1024 * 1024 * 1024); // default: 1 GiB

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .thread_name(|i| format!("oxidd mi {i}")) // "mi" for "manager index"
            .stack_size(stack_size)
            .build()
            .expect("could not build thread pool");
        let split_depth = AtomicU32::new(Workers::auto_split_depth(&pool));
        Self { pool, split_depth }
    }
    #[cfg(feature = "lace")]
    pub(crate) fn new(threads: u32) -> Self {
        // stack size is not (yet) configurable in Lace
        // let pool = Mutex::new(lace::Lace::init(threads as usize));
        // split depth is not really used with Lace
        let split_depth = AtomicU32::new(0);
        let num_threads = threads as usize;
        Self { split_depth, num_threads }
    }

    fn auto_split_depth(pool: &rayon::ThreadPool) -> u32 {
        let threads = pool.current_num_threads();
        if threads > 1 {
            (4096 * threads).ilog2()
        } else {
            0
        }
    }
}

impl oxidd_core::WorkerPool for Workers {
    #[inline]
    fn current_num_threads(&self) -> usize {
        #[cfg(not(feature = "lace"))]
        {
            self.pool.current_num_threads()
        }
        #[cfg(feature = "lace")]
        {
            self.num_threads
        }
    }

    #[inline(always)]
    fn split_depth(&self) -> u32 {
        self.split_depth.load(Relaxed)
    }

    fn set_split_depth(&self, depth: Option<u32>) {
        #[cfg(not(feature = "lace"))]
        {
            let depth = match depth {
                Some(d) => d,
                None => Self::auto_split_depth(&self.pool),
            };
            self.split_depth.store(depth, Relaxed);
        }
        #[cfg(feature = "lace")]
        {
            // setting split depth is not supported with Lace
        }
    }

    #[inline]
    fn install<RA: Send>(&self, op: impl FnOnce() -> RA + Send) -> RA {
        #[cfg(not(feature = "lace"))]
        {
            self.pool.install(op)
        }
        #[cfg(feature = "lace")]
        {
            // installing is not yet supported with Lace
            op()
        }
    }

    #[inline]
    fn join<RA: Send, RB: Send>(
        &self,
        op_a: impl FnOnce() -> RA + Send,
        op_b: impl FnOnce() -> RB + Send,
    ) -> (RA, RB) {
        #[cfg(not(feature = "lace"))]
        {
            self.pool.join(op_a, op_b)
        }
        #[cfg(feature = "lace")]
        {
            // joining is not yet supported with Lace
            (op_a(), op_b())
        }
    }

    #[inline]
    fn broadcast<RA: Send>(
        &self,
        op: impl Fn(oxidd_core::BroadcastContext) -> RA + Sync,
    ) -> Vec<RA> {
        #[cfg(not(feature = "lace"))]
        {
            self.pool.broadcast(|ctx| {
                op(oxidd_core::BroadcastContext {
                    index: ctx.index() as u32,
                    num_threads: ctx.num_threads() as u32,
                })
            })
        }
        #[cfg(feature = "lace")]
        {
            // broadcasting is not yet supported with Lace
            let num_threads = self.current_num_threads() as u32;
            (0..num_threads)
                .map(|index| op(oxidd_core::BroadcastContext { index, num_threads }))
                .collect()
        }
    }
}