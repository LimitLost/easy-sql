use super::syntax::{GetTrait, GetTraitTable, GetTraitTest, GetTraitTest2};
#[allow(dead_code)]
fn get_test() {
    let _r: GetTraitTest = GetTraitTable::get();
    let _r2: GetTraitTest2 = GetTraitTable::get();
}
