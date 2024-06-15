#[derive(Debug)]
pub struct Tabs(Vec<usize>);

impl Tabs {
    pub fn new(cols: usize) -> Self {
        let mut tabs = vec![];
        for t in (8..cols).step_by(8) {
            tabs.push(t);
        }
        Self(tabs)
    }

    pub fn set(&mut self, pos: usize) {
        if let Err(index) = self.0.binary_search(&pos) {
            self.0.insert(index, pos);
        }
    }

    pub fn unset(&mut self, pos: usize) {
        if let Ok(index) = self.0.binary_search(&pos) {
            self.0.remove(index);
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn before(&self, pos: usize, n: usize) -> Option<usize> {
        self.0
            .iter()
            .rev()
            .skip_while(|t| pos <= **t)
            .nth(n - 1)
            .copied()
    }

    pub fn after(&self, pos: usize, n: usize) -> Option<usize> {
        self.0.iter().skip_while(|t| pos >= **t).nth(n - 1).copied()
    }
}
