use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
#[cfg(feature = "lace")]
use std::sync::Mutex;

/// Worker thread pool
pub struct Workers {
    #[cfg(not(feature = "lace"))]
    pub(crate) pool: rayon::ThreadPool,
    #[cfg(feature = "lace")]
    pub lace: Mutex<lace::Lace>,
    #[cfg(feature = "lace")]
    lace_pool_id: usize,
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
            .thread_name(|i| format!("oxidd mp {i}")) // "mp" for "manager pointer"
            .stack_size(stack_size)
            .build()
            .expect("could not build thread pool");
        let split_depth = AtomicU32::new(Workers::auto_split_depth(&pool));
        Self { pool, split_depth }
    }

    #[cfg(feature = "lace")]
    pub(crate) fn new(threads: u32) -> Self {
        // stack size is not (yet) configurable in Lace
        let split_depth = AtomicU32::new(0);
        let num_threads = threads as usize;
        let lace = lace::Lace::init(num_threads);
        let lace_pool_id = lace.pool_id();
        Self { lace: Mutex::new(lace), lace_pool_id, split_depth, num_threads }
    }

    #[cfg(not(feature = "lace"))]
    fn auto_split_depth(pool: &rayon::ThreadPool) -> u32 {
        let threads = pool.current_num_threads();
        if threads > 1 {
            (4096 * threads).ilog2()
        } else {
            0
        }
    }
}

#[cfg(not(feature = "lace"))]
impl oxidd_core::WorkerPool for Workers {
    type Context = ();

    #[inline]
    fn current_num_threads(&self) -> usize {
        self.pool.current_num_threads()
    }

    #[inline(always)]
    fn split_depth(&self) -> u32 {
        self.split_depth.load(Relaxed)
    }

    fn set_split_depth(&self, depth: Option<u32>) {
        let depth = match depth {
            Some(d) => d,
            None => Self::auto_split_depth(&self.pool),
        };
        self.split_depth.store(depth, Relaxed);
    }

    #[inline]
    fn install<I, O>(
        &self,
        task: oxidd_core::WorkerTask<(), I, O>,
        input: I,
    ) -> O
    where
        I: Send,
        O: Send,
    {
        self.pool.install(move || {
            let mut cx = ();
            task(&mut cx, input)
        })
    }

    #[inline]
    fn join<IA, IB, OA, OB>(
        &self,
        _cx: &mut (),
        task_a: oxidd_core::WorkerTask<(), IA, OA>,
        input_a: IA,
        task_b: oxidd_core::WorkerTask<(), IB, OB>,
        input_b: IB,
    ) -> (OA, OB)
    where
        IA: Send,
        IB: Send,
        OA: Send,
        OB: Send,
    {
        self.pool.join(
            move || {
                let mut cx = ();
                task_a(&mut cx, input_a)
            },
            move || {
                let mut cx = ();
                task_b(&mut cx, input_b)
            },
        )
    }

    #[inline]
    fn broadcast<R: Send>(
        &self,
        op: impl Fn(oxidd_core::BroadcastContext) -> R + Sync,
    ) -> Vec<R> {
        self.pool.broadcast(|ctx| {
            op(oxidd_core::BroadcastContext {
                index: ctx.index() as u32,
                num_threads: ctx.num_threads() as u32,
            })
        })
    }
}

#[cfg(feature = "lace")]
impl oxidd_core::WorkerPool for Workers {
    type Context = lace::Worker;

    #[inline]
    fn current_num_threads(&self) -> usize {
        self.num_threads
    }

    #[inline(always)]
    fn split_depth(&self) -> u32 {
        self.split_depth.load(Relaxed)
    }

    fn set_split_depth(&self, depth: Option<u32>) {
        if let Some(depth) = depth {
            self.split_depth.store(depth, Relaxed);
        }
    }

    #[inline]
    fn install<I, O>(
        &self,
        task: oxidd_core::WorkerTask<lace::Worker, I, O>,
        input: I,
    ) -> O
    where
        I: Send,
        O: Send,
    {
        match lace::try_run_current(self.lace_pool_id, task, input) {
            Ok(output) => output,
            Err(input) => {
                let mut lace = self.lace.lock().unwrap();

                lace.run(|worker| task(worker, input))
            }
        }
    }

    #[inline]
    fn join<IA, IB, OA, OB>(
        &self,
        worker: &mut lace::Worker,
        task_a: oxidd_core::WorkerTask<lace::Worker, IA, OA>,
        input_a: IA,
        task_b: oxidd_core::WorkerTask<lace::Worker, IB, OB>,
        input_b: IB,
    ) -> (OA, OB)
    where
        IA: Send,
        IB: Send,
        OA: Send,
        OB: Send,
    {
        worker.join(task_a, input_a, task_b, input_b)
    }

    #[inline]
    fn broadcast<R: Send>(
        &self,
        op: impl Fn(oxidd_core::BroadcastContext) -> R + Sync,
    ) -> Vec<R> {
        let mut lace = self.lace.lock().unwrap();
        lace.broadcast(|ctx| {
            op(oxidd_core::BroadcastContext {
                index: ctx.index() as u32,
                num_threads: ctx.num_workers() as u32,
            })
        })
    }
}