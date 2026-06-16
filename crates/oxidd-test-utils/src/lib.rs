//! Mock implementations of core traits and other testing utilities

#![warn(missing_docs)]

use oxidd_core::{BroadcastContext, WorkerPool};

pub mod edge;

/// Worker thread pool (implemented via rayon)
pub struct Workers;

impl WorkerPool for Workers {
    type Context = ();

    fn current_num_threads(&self) -> usize {
        rayon::current_num_threads()
    }

    fn split_depth(&self) -> u32 {
        42
    }

    fn set_split_depth(&self, _depth: Option<u32>) {}

    fn install<I, O>(
        &self,
        task: oxidd_core::WorkerTask<(), I, O>,
        input: I,
    ) -> O
    where
        I: Send,
        O: Send,
    {
        let mut cx = ();
        task(&mut cx, input)
    }

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
        rayon::join(
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

    fn broadcast<R: Send>(&self, op: impl Fn(oxidd_core::BroadcastContext) -> R + Sync) -> Vec<R> {
        rayon::broadcast(|ctx| {
            op(BroadcastContext {
                index: ctx.index() as u32,
                num_threads: ctx.num_threads() as u32,
            })
        })
    }
}
