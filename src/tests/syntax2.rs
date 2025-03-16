use super::syntax::{GetTrait, GetTraitTable, GetTraitTest, GetTraitTest2};

fn get_test() {
    let r: GetTraitTest = GetTraitTable::get();
    let r2: GetTraitTest2 = GetTraitTable::get();
}
