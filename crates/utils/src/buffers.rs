#[derive(Clone)]
pub struct Buffers<const LEN: usize> {
    pub r: [f32; LEN],
    pub l: [f32; LEN],
    pub idx: usize,
}
// unsafe impl<const LEN: usize> Sync for Buffers<LEN> {}
// unsafe impl<const LEN: usize> Send for Buffers<LEN> {}

impl<const LEN: usize> Buffers<LEN> {
    pub const fn new() -> Self {
        Self {
            l: [0.; LEN],
            r: [0.; LEN],
            idx: 0,
        }
    }

    pub const fn from(l: [f32; LEN], r: [f32; LEN]) -> Self {
        Self { l, r, idx: 0 }
    }

    /// Writes to buffer, advances idx, returns true if buffer is full
    pub fn write_advance(&mut self, l: f32, r: f32) -> bool {
        self.l[self.idx] = l;
        self.r[self.idx] = r;

        self.idx += 1;
        if self.idx >= LEN {
            self.idx = 0;
            true
        } else {
            false
        }
    }

    /// Reads buffer, advances idx, returns true if buffer is full
    pub fn read_advance(&mut self) -> (f32, f32, bool) {
        (self.l[self.idx], self.r[self.idx], {
            self.idx += 1;
            if self.idx >= LEN {
                self.idx = 0;
                true
            } else {
                false
            }
        })
    }

    /// Reads buffer, advances idx, returns true if buffer is full
    pub fn read(&self) -> (f32, f32) {
        (self.l[self.idx], self.r[self.idx])
    }

    pub fn read_at(&self, idx: usize) -> (f32, f32) {
        let idx = idx % LEN;
        (self.l[idx], self.r[idx])
    }

    pub fn reset(&mut self) {
        self.idx = 0;
    }
}
