use crate::system::{SystemInput, SystemParam};



pub trait SystemFunction: Send + Sync + 'static {
    type Input: Sized;
    type Param: SystemParam;
    type Out;

    fn run(
        &mut self,
        input: SystemInput<Self::Input>,
        param: <Self::Param as SystemParam>::Item<'_, '_>,
    ) -> Self::Out;
}



