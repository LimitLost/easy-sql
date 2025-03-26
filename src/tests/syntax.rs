use easy_macros::macros::always_context;

#[always_context]
pub trait GetTrait<T> {
    fn get() -> T;
    fn get_lazy() -> T;
}

pub struct GetTraitTable;
pub struct GetTraitTable2;

pub struct GetTraitTest;
pub struct GetTraitTest2;

#[always_context]
impl GetTrait<GetTraitTest> for GetTraitTable {
    fn get() -> GetTraitTest {
        GetTraitTest
    }
    fn get_lazy() -> GetTraitTest {
        GetTraitTest
    }
}
#[always_context]
impl GetTrait<GetTraitTest> for GetTraitTable2 {
    fn get() -> GetTraitTest {
        GetTraitTest
    }
    fn get_lazy() -> GetTraitTest {
        GetTraitTest
    }
}

#[always_context]
impl GetTrait<GetTraitTest2> for GetTraitTable {
    fn get() -> GetTraitTest2 {
        GetTraitTest2
    }
    fn get_lazy() -> GetTraitTest2 {
        GetTraitTest2
    }
}

#[always_context]
impl GetTrait<GetTraitTest2> for GetTraitTable2 {
    fn get() -> GetTraitTest2 {
        GetTraitTest2
    }
    fn get_lazy() -> GetTraitTest2 {
        GetTraitTest2
    }
}
