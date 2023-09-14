
mod blog;
pub use crate::blog::Post;

fn main() {
    let mut post = Post::new();
    post.add_text("I ate a salad for lunch today");
    let post = post.request_review();
    assert_eq!("I ate a salad for lunch today", post.content());
}