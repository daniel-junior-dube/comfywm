use wlroots::{Area, Origin, Size, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle};

use std::collections::LinkedList;
use std::slice::IterMut;
use windows::WindowData;

/*
..####....####...##..##..######...####...######..##..##..######..#####..
.##..##..##..##..###.##....##....##..##....##....###.##..##......##..##.
.##......##..##..##.###....##....######....##....##.###..####....#####..
.##..##..##..##..##..##....##....##..##....##....##..##..##......##..##.
..####....####...##..##....##....##..##..######..##..##..######..##..##.
........................................................................
.#####....####...######...####..
.##..##..##..##....##....##..##.
.##..##..######....##....######.
.##..##..##..##....##....##..##.
.#####...##..##....##....##..##.
................................
*/

#[derive(Clone)]
pub enum ContainerAxis {
	Vertical,
	Horizontal,
}

#[derive(Clone)]
pub struct ContainerData {
	pub children_indices: Vec<usize>,
	pub area: Area,
	pub axis: ContainerAxis,
}

impl ContainerData {
	fn new(children_indices: Vec<usize>, axis: ContainerAxis, area: Area) -> Self {
		ContainerData {
			children_indices,
			axis,
			area,
		}
	}

	fn new_empty(area: Area) -> Self {
		ContainerData {
			children_indices: vec![],
			axis: ContainerAxis::Horizontal,
			area,
		}
	}

	fn new_with_children(children_indices: Vec<usize>, area: Area) -> Self {
		ContainerData {
			children_indices,
			axis: ContainerAxis::Horizontal,
			area,
		}
	}

	fn add_child(&mut self, child_index: usize) {
		self.children_indices.push(child_index);
	}

	fn change_axis(&mut self, new_axis: ContainerAxis) {
		self.axis = new_axis;
	}
}

/*
.##.......####...##..##...####...##..##..######..##..##...####...#####...######.
.##......##..##...####...##..##..##..##....##....###.##..##..##..##..##..##.....
.##......######....##....##..##..##..##....##....##.###..##..##..##..##..####...
.##......##..##....##....##..##..##..##....##....##..##..##..##..##..##..##.....
.######..##..##....##.....####....####.....##....##..##...####...#####...######.
................................................................................
*/

pub enum LayoutNodeType {
	Container(ContainerData),
	Window(WindowData),
}

pub struct LayoutNode {
	node_type: LayoutNodeType,
	parent_node_index: usize,
	weight: f32,
	// TODO: Add Area to each node instead of in window data and container data
	//area: Area,
}

impl LayoutNode {
	pub fn new_root_node(area: Area) -> Self {
		let container_data = ContainerData::new_empty(area);
		LayoutNode::new_container_node(container_data, 0)
	}

	pub fn new_window_node(window_data: WindowData, parent_node_index: usize) -> Self {
		LayoutNode {
			node_type: LayoutNodeType::Window(window_data),
			parent_node_index,
			weight: 1.0,
		}
	}

	pub fn new_container_node(container_node_data: ContainerData, parent_node_index: usize) -> Self {
		LayoutNode {
			node_type: LayoutNodeType::Container(container_node_data),
			parent_node_index,
			weight: 1.0,
		}
	}

	pub fn is_node_containing_shell_handle(&self, shell_handle: &WLRXdgV6ShellSurfaceHandle) -> bool {
		match self.node_type {
			LayoutNodeType::Window(ref window_data) => window_data.shell_handle == *shell_handle,
			_ => false,
		}
	}
}

/*
.##.......####...##..##...####...##..##..######.
.##......##..##...####...##..##..##..##....##...
.##......######....##....##..##..##..##....##...
.##......##..##....##....##..##..##..##....##...
.######..##..##....##.....####....####.....##...
................................................
*/

pub struct Layout {
	pub available_places: LinkedList<usize>,
	pub nodes: Vec<Option<LayoutNode>>,
	pub active_node_index: usize,
	pub nb_windows: usize,
}

impl Layout {
	pub fn new(output_area: Area) -> Self {
		let root_node = LayoutNode::new_root_node(output_area);
		Layout {
			available_places: LinkedList::<usize>::new(),
			nodes: vec![Some(root_node)],
			active_node_index: 0,
			nb_windows: 0,
		}
	}

	pub fn root_node(&self) -> Option<&LayoutNode> {
		if let Some(ref root_node) = self.nodes[0] {
			Some(root_node)
		} else {
			None
		}
	}

	pub fn render_box(&self) -> Option<Area> {
		if let Some(root_node) = self.root_node() {
			match root_node.node_type {
				LayoutNodeType::Container(ContainerData { area, .. }) => Some(area),
				_ => None,
			}
		} else {
			None
		}
	}

	pub fn windows_data(&self) -> Vec<WindowData> {
		self
			.nodes
			.iter()
			.filter_map(|element| {
				if let Some(LayoutNode { node_type, .. }) = element {
					if let LayoutNodeType::Window(ref window_data) = node_type {
						return Some(window_data);
					}
				}
				None
			}).cloned()
			.collect()
	}

	pub fn parent_node_index_of(&self, node_index: usize) -> Option<usize> {
		if let Some(Some(node)) = &self.nodes.get(node_index) {
			Some(node.parent_node_index)
		} else {
			None
		}
	}

	pub fn parent_node_index_of_active_node(&self) -> usize {
		if self.active_node_index == 0 {
			return 0;
		}
		match self.parent_node_index_of(self.active_node_index) {
			Some(parent_node_index) => parent_node_index,
			None => 0,
		}
	}

	pub fn add_node(&mut self, layout_node: LayoutNode) -> usize {
		let node_index = if let Some(available_index) = self.available_places.pop_back() {
			self.nodes[available_index] = Some(layout_node);
			available_index
		} else {
			let temp_node_index = self.nodes.len();
			self.nodes.push(Some(layout_node));
			temp_node_index
		};
		node_index
	}

	pub fn get_container_data_of_node(&self, container_node_index: usize) -> Option<ContainerData> {
		if let Some(parent_node) = &self.nodes[container_node_index] {
			if let LayoutNodeType::Container(container_node_data) = &parent_node.node_type {
				return Some(container_node_data.clone());
			}
		}
		None
	}

	pub fn rebalance_container(&mut self, node_index: usize) {
		if let Some(container_data) = self.get_container_data_of_node(node_index) {
			let children_weight_sum: f32 = container_data
				.children_indices
				.iter()
				.map(|index| match &self.nodes[*index] {
					Some(layout_node) => layout_node.weight,
					None => 0.0,
				}).sum();

			let mut origin_offset: i32 = 0;
			for index in container_data.children_indices {
				if let Some(Some(layout_node)) = self.nodes.iter_mut().nth(index) {
					match &mut layout_node.node_type {
						LayoutNodeType::Window(ref mut window_data) => match container_data.axis {
							ContainerAxis::Horizontal => {
								let area_width =
									((layout_node.weight / children_weight_sum) * container_data.area.size.width as f32) as i32;
								let area_height = container_data.area.size.height;
								let new_area = Area::new(
									Origin::new(origin_offset, container_data.area.origin.y),
									Size::new(area_width, area_height),
								);
								window_data.area = new_area;
								origin_offset += area_width;
							}
							ContainerAxis::Vertical => {
								let area_width = container_data.area.size.height;
								let area_height =
									((layout_node.weight / children_weight_sum) * container_data.area.size.height as f32) as i32;
								let new_area = Area::new(
									Origin::new(container_data.area.origin.x, origin_offset),
									Size::new(area_width, area_height),
								);
								window_data.area = new_area;
								origin_offset += area_height;
							}
						},
						_ => {}
					}
				}
			}
		}
	}

	pub fn add_child_to_container_of_active_node(&mut self, node_index: usize) {
		let parent_node_index = self.parent_node_index_of_active_node();
		// ? Set parent to child
		if let Some(node) = &mut self.nodes[node_index] {
			node.parent_node_index = parent_node_index;
		}

		// ? Set child to parent
		if let Some(parent_node) = &mut self.nodes[parent_node_index] {
			if let LayoutNodeType::Container(ref mut container_node_data) = parent_node.node_type {
				container_node_data.add_child(node_index);
			}
		}

		// ? Rebalance parent container
		self.rebalance_container(parent_node_index);
	}

	pub fn add_window(&mut self, shell_handle: WLRXdgV6ShellSurfaceHandle) {
		let window_data = WindowData::new(shell_handle);
		let node_index = self.add_node(LayoutNode::new_window_node(window_data, 0));
		self.add_child_to_container_of_active_node(node_index);
		self.nb_windows += 1;
		self.active_node_index = node_index;
	}

	pub fn remove_node(&mut self, node_index: usize) {
		let mut should_remove_container = false;
		if let Some(parent_node_index) = self.parent_node_index_of(node_index) {
			if let Some(parent_node) = &mut self.nodes[parent_node_index] {
				if let LayoutNodeType::Container(ref mut container_node_data) = parent_node.node_type {
					let result = container_node_data
						.children_indices
						.iter()
						.position(|element| *element == node_index);
					if let Some(index_of_child) = result {
						container_node_data.children_indices.remove(index_of_child);
						should_remove_container = container_node_data.children_indices.len() == 0;
					}
				}
			}
			if should_remove_container {
				self.remove_node(parent_node_index);
			}
		}
	}

	pub fn remove_window(&mut self, shell_handle: &WLRXdgV6ShellSurfaceHandle) {
		let result = self.nodes.iter().position(|element| match *element {
			Some(ref node) => node.is_node_containing_shell_handle(shell_handle),
			None => false,
		});
		if let Some(index) = result {
			self.remove_node(index);
		}
	}
}
