use oxidd_core::Manager;
use oxidd_core::util::{AllocResult, Borrowed, EdgeDropGuard};

pub type UnaryInput<'a, M, R> = (&'a M, R, Borrowed<'a, <M as Manager>::Edge>);

pub type UnaryOp<'a, M, R> = oxidd_core::WorkerTask<
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

pub type BinaryOp<'a, M, R> = oxidd_core::WorkerTask<
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

pub type TernaryOp<'a, M, R> = oxidd_core::WorkerTask<
    <R as Recursor<M>>::Context,
    TernaryInput<'a, M, R>,
    AllocResult<<M as Manager>::Edge>,
>;

pub type SubstInput<'a, M, R> = (
    &'a M,
    R,
    Borrowed<'a, <M as Manager>::Edge>,
    &'a [<M as Manager>::Edge],
    u32,
);

pub type SubstOp<'a, M, R> = oxidd_core::WorkerTask<
    <R as Recursor<M>>::Context,
    SubstInput<'a, M, R>,
    AllocResult<<M as Manager>::Edge>,
>;

pub type ApplyQuantDispatchInput<'a, M, R> = (
    &'a M,
    R,
    oxidd_core::function::BooleanOperator,
    Borrowed<'a, <M as Manager>::Edge>,
    Borrowed<'a, <M as Manager>::Edge>,
    Borrowed<'a, <M as Manager>::Edge>,
);

pub trait Recursor<M>: Copy
where
    M: Manager,
{
    type Context;

    fn unary<'a>(
        self,
        op: UnaryOp<'a, M, Self>,
        manager: &'a M,
        a: Borrowed<'a, M::Edge>,
        b: Borrowed<'a, M::Edge>,
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)>;

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

    fn subst<'a>(
        self,
        op: SubstOp<'a, M, Self>,
        manager: &'a M,
        a: (Borrowed<'a, M::Edge>, &'a [M::Edge], u32),
        b: (Borrowed<'a, M::Edge>, &'a [M::Edge], u32),
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)>;

    /// Returns true if the algorithm should switch to a sequential recursor
    ///
    /// With the current [`join()`][oxidd_core::WorkerPool::join]
    /// implementations, we observe a significant performance overhead
    /// compared to sequentially calling the functions. Therefore, it may
    /// make sense to switch to the sequential version after, e.g., a
    /// certain recursion depth.
    fn should_switch_to_sequential(self) -> bool;
}

#[derive(Clone, Copy)]
pub struct SequentialRecursor;

impl<M> Recursor<M> for SequentialRecursor
where
    M: Manager,
{
    type Context = ();

    fn unary<'a>(
        self,
        op: UnaryOp<'a, M, Self>,
        manager: &'a M,
        a: Borrowed<'a, M::Edge>,
        b: Borrowed<'a, M::Edge>,
        cx: &mut Self::Context,
    ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
        let ra = op(cx, (manager, self, a))?;
        let rb = op(cx, (manager, self, b))?;

        Ok((
            EdgeDropGuard::new(manager, ra),
            EdgeDropGuard::new(manager, rb),
        ))
    }

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

    fn subst<'a>(
        self,
        op: SubstOp<'a, M, Self>,
        manager: &'a M,
        a: (Borrowed<'a, M::Edge>, &'a [M::Edge], u32),
        b: (Borrowed<'a, M::Edge>, &'a [M::Edge], u32),
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
        false
    }
}

#[cfg(any(feature = "multi-threading", feature = "lace"))]
pub mod mt {
    use super::*;
    use oxidd_core::{HasWorkers, WorkerPool};

    #[derive(Clone, Copy)]
    pub struct ParallelRecursor {
        remaining_depth: u32,
    }

    impl ParallelRecursor {
        pub fn new<M>(manager: &M) -> Self
        where
            M: HasWorkers,
        {
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

        fn unary<'a>(
            mut self,
            op: UnaryOp<'a, M, Self>,
            manager: &'a M,
            a: Borrowed<'a, M::Edge>,
            b: Borrowed<'a, M::Edge>,
            cx: &mut Self::Context,
        ) -> AllocResult<(EdgeDropGuard<'a, M>, EdgeDropGuard<'a, M>)> {
            self.remaining_depth -= 1;

            let (ra, rb) =
                manager
                    .workers()
                    .join(cx, op, (manager, self, a), op, (manager, self, b));

            Ok((
                EdgeDropGuard::new(manager, ra?),
                EdgeDropGuard::new(manager, rb?),
            ))
        }

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

        fn subst<'a>(
            mut self,
            op: SubstOp<'a, M, Self>,
            manager: &'a M,
            a: (Borrowed<'a, M::Edge>, &'a [M::Edge], u32),
            b: (Borrowed<'a, M::Edge>, &'a [M::Edge], u32),
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
