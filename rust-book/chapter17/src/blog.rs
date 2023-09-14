pub struct Post {
    content: String
}

impl Post {
    pub fn new () -> DraftPost {
        DraftPost {
            content: String::new()
        }
    }
}

pub struct DraftPost {
    content: String
}

impl DraftPost {
    pub fn request_review (self) -> PendingReview {
        PendingReview {
            content: self.content, // transfer ownership of content.
            num_approvals: 0,
        }
    }

    pub fn add_text(&mut self, text : &str) {
        self.content.push_str (text);
    }
}

pub struct PendingReview {
    content: String,
    num_approvals: i32
}

impl PendingReview {
    pub fn content(&self) -> &String {
        &self.content
    }
}

#[test]
fn test_review () {
    let mut p = Post::new();
    p.add_text("foo");
    // assert_eq!(p.content(), ""); // Expect to fail to compile.
    let p = p.request_review();
    assert_eq!(p.content(), "foo");
}   