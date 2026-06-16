use oxidd_core::util::{AllocResult, Borrowed, EdgeDropGuard};
use oxidd_core::{LevelNo, Manager, VarNo, WorkerTask};

pub type UnaryInput<'a, M, R> = (
    &'a M,
    R,
    Borrowed<'a, <M as Manager>::Edge>,
);

pub type UnaryOp<'a, M, R> = WorkerTask<
    <R as Recursor<M>>::Context,
    UnaryInput<'a, M, R>,
    AllocResult<<M as Manager>::Edge>,
>;

pub type BinaryInput<'a, M, R> = (
    &'a M,
    R,
    Borrowed<'a, <M as Manager>::Edge>,
    Borrowed<'a, <M as Manager>::Edge>,
);

pub type BinaryOp<'a, M, R> = WorkerTask<
    <R as Recursor<M>>::Context,
    BinaryInput<'a, M, R>,
    AllocResult<<M as Manager>::Edge>,
>;

pub type TernaryInput<'a, M, R> = (
    &'a M,
    R,
    Borrowed<'a, <M as Manager>::Edge>,
    Borrowed<'a, <M as Manager>::Edge>,
    Borrowed<'a, <M as Manager>::Edge>,
);

pub type TernaryOp<'a, M, R> = WorkerTask<
    <R as Recursor<M>>::Context,
    TernaryInput<'a, M, R>,
    AllocResult<<M as Manager>::Edge>,
>;

pub type SubsetInput<'a, M, R> = (
    &'a M,
    R,
    Borrowed<'a, <M as Manager>::Edge>,
    VarNo,
    LevelNo,
);

pub type SubsetOp<'a, M, R> = WorkerTask<
    <R as Recursor<M>>::Context,
    SubsetInput<'a, M, R>,
    AllocResult<<M as Manager>::Edge>,
>;

pub trait Recursor<M: Manager>: Copy {
    type Context;

    fn binary<'a>(
        self,
        op: BinaryOp<'a, M, Self>,
        manager: &'a M,
        a: (Borrowed<'a, M::Edge>, Borrowed<'a, M::Edge>),
        b: (Borrowed<'a, M::Edge>, Borrowed<'a, M::Edge>),
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)>;

    #[allow(clippy::type_complexity)]
    fn ternary<'a>(
        self,
        op: TernaryOp<'a, M, Self>,
        manager: &'a M,
        a: (
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
        ),
        b: (
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
        ),
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)>;

    #[allow(clippy::type_complexity)]
    fn binary_ternary<'a>(
        self,
        manager: &'a M,
        op_a: BinaryOp<'a, M, Self>,
        a: (Borrowed<'a, M::Edge>, Borrowed<'a, M::Edge>),
        op_b: TernaryOp<'a, M, Self>,
        b: (
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
        ),
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)>;

    fn subset<'a>(
        self,
        op: SubsetOp<'a, M, Self>,
        manager: &'a M,
        a: (Borrowed<'a, M::Edge>, VarNo, LevelNo),
        b: (Borrowed<'a, M::Edge>, VarNo, LevelNo),
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)>;

    /// Returns true if the algorithm should switch to a sequential recursor.
    fn should_switch_to_sequential(self) -> bool;
}

#[derive(Clone, Copy)]
pub struct SequentialRecursor;

impl<M: Manager> Recursor<M> for SequentialRecursor {
    type Context = ();

    #[inline(always)]
    fn binary<'a>(
        self,
        op: BinaryOp<'a, M, Self>,
        manager: &'a M,
        a: (Borrowed<'a, M::Edge>, Borrowed<'a, M::Edge>),
        b: (Borrowed<'a, M::Edge>, Borrowed<'a, M::Edge>),
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
        let ra = op(cx, (manager, self, a.0, a.1))?;
        let rb = op(cx, (manager, self, b.0, b.1))?;

        Ok((
            EdgeDropGuard::new(manager, ra),
            EdgeDropGuard::new(manager, rb),
        ))
    }

    #[inline(always)]
    fn ternary<'a>(
        self,
        op: TernaryOp<'a, M, Self>,
        manager: &'a M,
        a: (
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
        ),
        b: (
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
        ),
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
        let ra = op(cx, (manager, self, a.0, a.1, a.2))?;
        let rb = op(cx, (manager, self, b.0, b.1, b.2))?;

        Ok((
            EdgeDropGuard::new(manager, ra),
            EdgeDropGuard::new(manager, rb),
        ))
    }

    #[inline(always)]
    fn binary_ternary<'a>(
        self,
        manager: &'a M,
        op_a: BinaryOp<'a, M, Self>,
        a: (Borrowed<'a, M::Edge>, Borrowed<'a, M::Edge>),
        op_b: TernaryOp<'a, M, Self>,
        b: (
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
            Borrowed<'a, M::Edge>,
        ),
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
        let ra = op_a(cx, (manager, self, a.0, a.1))?;
        let rb = op_b(cx, (manager, self, b.0, b.1, b.2))?;

        Ok((
            EdgeDropGuard::new(manager, ra),
            EdgeDropGuard::new(manager, rb),
        ))
    }

    #[inline(always)]
    fn subset<'a>(
        self,
        op: SubsetOp<'a, M, Self>,
        manager: &'a M,
        a: (Borrowed<'a, M::Edge>, VarNo, LevelNo),
        b: (Borrowed<'a, M::Edge>, VarNo, LevelNo),
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
        let ra = op(cx, (manager, self, a.0, a.1, a.2))?;
        let rb = op(cx, (manager, self, b.0, b.1, b.2))?;

        Ok((
            EdgeDropGuard::new(manager, ra),
            EdgeDropGuard::new(manager, rb),
        ))
    }

    #[inline(always)]
    fn should_switch_to_sequential(self) -> bool {
        false // true would make the algorithms diverge
    }
}

#[cfg(feature = "multi-threading")]
pub mod mt {
    use super::*;
    use oxidd_core::{HasWorkers, WorkerPool};

    #[derive(Clone, Copy)]
    pub struct ParallelRecursor {
        remaining_depth: u32,
    }

    impl ParallelRecursor {
        pub fn new<M: HasWorkers>(manager: &M) -> Self {
            Self {
                remaining_depth: manager.workers().split_depth(),
            }
        }
    }

    impl<M> Recursor<M> for ParallelRecursor
    where
        M: Manager + HasWorkers,
        M::Edge: Send + Sync,
    {
        type Context = <<M as HasWorkers>::WorkerPool as WorkerPool>::Context;

        fn binary<'a>(
            mut self,
            op: BinaryOp<'a, M, Self>,
            manager: &'a M,
            a: (Borrowed<'a, M::Edge>, Borrowed<'a, M::Edge>),
            b: (Borrowed<'a, M::Edge>, Borrowed<'a, M::Edge>),
            cx: &mut Self::Context,
        ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
            self.remaining_depth -= 1;

            let (ra, rb) = manager.workers().join(
                cx,
                op,
                (manager, self, a.0, a.1),
                op,
                (manager, self, b.0, b.1),
            );

            Ok((
                EdgeDropGuard::new(manager, ra?),
                EdgeDropGuard::new(manager, rb?),
            ))
        }

        fn ternary<'a>(
            mut self,
            op: TernaryOp<'a, M, Self>,
            manager: &'a M,
            a: (
                Borrowed<'a, M::Edge>,
                Borrowed<'a, M::Edge>,
                Borrowed<'a, M::Edge>,
            ),
            b: (
                Borrowed<'a, M::Edge>,
                Borrowed<'a, M::Edge>,
                Borrowed<'a, M::Edge>,
            ),
            cx: &mut Self::Context,
        ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
            self.remaining_depth -= 1;

            let (ra, rb) = manager.workers().join(
                cx,
                op,
                (manager, self, a.0, a.1, a.2),
                op,
                (manager, self, b.0, b.1, b.2),
            );

            Ok((
                EdgeDropGuard::new(manager, ra?),
                EdgeDropGuard::new(manager, rb?),
            ))
        }

        #[inline(always)]
        fn binary_ternary<'a>(
            mut self,
            manager: &'a M,
            op_a: BinaryOp<'a, M, Self>,
            a: (Borrowed<'a, M::Edge>, Borrowed<'a, M::Edge>),
            op_b: TernaryOp<'a, M, Self>,
            b: (
                Borrowed<'a, M::Edge>,
                Borrowed<'a, M::Edge>,
                Borrowed<'a, M::Edge>,
            ),
            cx: &mut Self::Context,
        ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
            self.remaining_depth -= 1;

            let (ra, rb) = manager.workers().join(
                cx,
                op_a,
                (manager, self, a.0, a.1),
                op_b,
                (manager, self, b.0, b.1, b.2),
            );

            Ok((
                EdgeDropGuard::new(manager, ra?),
                EdgeDropGuard::new(manager, rb?),
            ))
        }

        fn subset<'a>(
            mut self,
            op: SubsetOp<'a, M, Self>,
            manager: &'a M,
            a: (Borrowed<'a, M::Edge>, VarNo, LevelNo),
            b: (Borrowed<'a, M::Edge>, VarNo, LevelNo),
            cx: &mut Self::Context,
        ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
            self.remaining_depth -= 1;

            let (ra, rb) = manager.workers().join(
                cx,
                op,
                (manager, self, a.0, a.1, a.2),
                op,
                (manager, self, b.0, b.1, b.2),
            );

            Ok((
                EdgeDropGuard::new(manager, ra?),
                EdgeDropGuard::new(manager, rb?),
            ))
        }

        #[inline(always)]
        fn should_switch_to_sequential(self) -> bool {
            self.remaining_depth == 0
        }
    }
}