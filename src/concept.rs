// aarch64
pub struct Asm {
    post_ops: Vec<PostOp>,
    buffer: Vec<u8>,
}

// TODO make label generic (Hash)
pub enum PostOp {
    Branch { offset: usize, label: String },
    BranchWithLink { offset: usize, label: String },
}
