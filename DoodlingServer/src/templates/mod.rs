
#[macro_export]
macro_rules! include_template {
    ($name : literal) => {
        std::include_str!{concat!{std::env!{"CARGO_MANIFEST_DIR"},"/DoodlingHtmx/templates/",$name,".html"}}
    };
}