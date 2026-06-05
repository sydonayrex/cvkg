pub mod a {
    pub struct Foo;
    pub mod b {
        impl super::Foo {
            pub fn bar(&self) {}
        }
    }
}
pub mod c {
    pub fn do_it(f: &crate::a::Foo) {
        f.bar();
    }
}
fn main() {}
