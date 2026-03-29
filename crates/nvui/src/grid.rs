#![allow(dead_code)]

use nvim::GridCell as NvimGridCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCell {
	pub ch: char,
	pub hl_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
	pub width: usize,
	pub height: usize,
	pub cells: Vec<GridCell>,
}

impl Grid {
	pub fn new(width: usize, height: usize) -> Self {
		let mut grid = Self { width: 0, height: 0, cells: Vec::new() };
		grid.resize(width, height);
		grid
	}

	pub fn resize(&mut self, width: usize, height: usize) {
		self.width = width;
		self.height = height;
		self.cells.resize(self.cell_count(), GridCell { ch: ' ', hl_id: None });
		self.clear();
	}

	pub fn clear(&mut self) {
		for cell in &mut self.cells {
			*cell = GridCell { ch: ' ', hl_id: None };
		}
	}

	pub fn set_line(&mut self, row: usize, col_start: usize, cells: &[NvimGridCell]) {
		if row >= self.height || self.width == 0 {
			return;
		}

		let mut col = col_start;

		for cell in cells {
			let repeat = cell.repeat.unwrap_or(1) as usize;
			for _ in 0..repeat {
				if col >= self.width {
					return;
				}
				let index = self.cell_index(row, col);
				self.cells[index] = GridCell { ch: cell.text, hl_id: cell.hl_id };
				col += 1;
			}
		}
	}

	fn cell_index(&self, row: usize, col: usize) -> usize {
		(row * self.width) + col
	}

	fn cell_count(&self) -> usize {
		self.width * self.height
	}
}

#[cfg(test)]
mod tests {
	use super::{Grid, GridCell};
	use nvim::GridCell as NvimGridCell;

	#[test]
	fn new_grid() {
		let grid = Grid::new(3, 2);

		assert_eq!(grid.width, 3);
		assert_eq!(grid.height, 2);
		assert_eq!(grid.cells.len(), 6);
		assert!(grid.cells.iter().all(|cell| *cell == GridCell { ch: ' ', hl_id: None }));
	}

	#[test]
	fn clear() {
		let mut grid = Grid::new(2, 2);

		grid.cells[0] = GridCell { ch: 'x', hl_id: Some(1) };
		grid.cells[3] = GridCell { ch: 'y', hl_id: Some(2) };
		grid.clear();

		assert!(grid.cells.iter().all(|cell| *cell == GridCell { ch: ' ', hl_id: None }));
	}

	#[test]
	fn set_line() {
		let mut grid = Grid::new(5, 2);
		let cells = vec![
			NvimGridCell { text: 'a', hl_id: Some(1), repeat: Some(2) },
			NvimGridCell { text: 'b', hl_id: None, repeat: None },
			NvimGridCell { text: 'c', hl_id: Some(3), repeat: Some(10) },
		];

		grid.set_line(1, 1, &cells);

		let expected = vec![
			GridCell { ch: ' ', hl_id: None },
			GridCell { ch: ' ', hl_id: None },
			GridCell { ch: ' ', hl_id: None },
			GridCell { ch: ' ', hl_id: None },
			GridCell { ch: ' ', hl_id: None },
			GridCell { ch: ' ', hl_id: None },
			GridCell { ch: 'a', hl_id: Some(1) },
			GridCell { ch: 'a', hl_id: Some(1) },
			GridCell { ch: 'b', hl_id: None },
			GridCell { ch: 'c', hl_id: Some(3) },
		];

		assert_eq!(grid.cells, expected);
	}
}
