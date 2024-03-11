#[derive(Debug, Clone, PartialEq)]
pub struct Pagination<T> {
    pub pages: Vec<T>,
    pub page: usize,
    pub active: bool,
}

impl<T> Pagination<T> {
    pub fn new(pages: Vec<T>) -> Self {
        Self {
            pages,
            page: 0,
            active: true,
        }
    }

    pub fn get_current_page(&self) -> &T {
        self.pages.get(self.page).unwrap()
    }

    pub fn get_current_page_mut(&mut self) -> &mut T {
        self.pages.get_mut(self.page).unwrap()
    }

    pub fn goto_previous_page(&mut self) {
        if self.page == 0 {
            self.page = self.pages.len() - 1;
        } else {
            self.page -= 1;
        }
    }

    pub fn goto_next_page(&mut self) {
        self.page = (self.page + 1) % self.pages.len();
    }
}
