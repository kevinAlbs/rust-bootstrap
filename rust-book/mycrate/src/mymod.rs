use self::submod::submodfn;

mod submod;
pub fn test() -> i32 {
    submodfn()
}
fn testprivate() -> i32 {
    123
}
