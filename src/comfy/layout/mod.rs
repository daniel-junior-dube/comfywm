use wlroots::{Area, Origin, Size, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle, XdgV6ShellState as WLRXdgV6ShellState};

use std::collections::LinkedList;

use compositor::ComfyKernel;

/*
.##...##..######..##..##..#####....####...##...##..#####....####...######...####..
.##...##....##....###.##..##..##..##..##..##...##..##..##..##..##....##....##..##.
.##.#.##....##....##.###..##..##..##..##..##.#.##..##..##..######....##....######.
.#######....##....##..##..##..##..##..##..#######..##..##..##..##....##....##..##.
..##.##...######..##..##..#####....####....##.##...#####...##..##....##....##..##.
..................................................................................
*/

pub struct WindowData {
	pub shell_handle: WLRXdgV6ShellSurfaceHandle,
	pub area: Area,
}

impl WindowData {
	pub fn new(shell_handle: WLRXdgV6ShellSurfaceHandle, area: Area) -> Self {
		WindowData { shell_handle, area }
	}
}

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

#[derive(Clone, Eq, PartialEq)]
pub enum ContainerAxis {
	Vertical,
	Horizontal,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ContainerData {
	pub children_indices: Vec<usize>,
	pub axis: ContainerAxis,
}

impl ContainerData {
	fn new(children_indices: Vec<usize>, axis: ContainerAxis) -> Self {
		ContainerData { children_indices, axis }
	}

	fn new_empty() -> Self {
		ContainerData::new(vec![], ContainerAxis::Vertical)
	}

	fn new_with_children(children_indices: Vec<usize>) -> Self {
		ContainerData::new(children_indices, ContainerAxis::Horizontal)
	}

	fn add_child(&mut self, child_index: usize) {
		self.children_indices.push(child_index);
	}

	fn change_axis(&mut self, new_axis: ContainerAxis) {
		self.axis = new_axis;
	}

	fn get_children_weight_sum(&self, nodes: &Vec<Option<LayoutNode>>) -> f32 {
		self.children_indices
			.iter()
			.map(|index| match &nodes[*index] {
				Some(layout_node) => layout_node.weight,
				None => 0.0,
			}).sum()
	}

	fn index_of(&self, node_index: usize) -> Option<usize> {
		self
			.children_indices
			.iter()
			.position(|element| *element == node_index)
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

#[derive(PartialEq, Eq)]
pub enum LayoutNodeType {
	Container(ContainerData),
	Window(WLRXdgV6ShellSurfaceHandle),
}

pub struct LayoutNode {
	node_type: LayoutNodeType,
	parent_node_index: usize,
	weight: f32,
	area: Area,
}

impl LayoutNode {
	pub fn new_root_node(area: Area) -> Self {
		let mut node = LayoutNode::new_container_node(ContainerData::new_empty(), 0);
		node.area = area;
		node
	}

	pub fn new(layout_node_type: LayoutNodeType, parent_node_index: usize) -> Self {
		LayoutNode {
			node_type: layout_node_type,
			parent_node_index,
			weight: 1.0,
			area: Area::new(Origin::new(0, 0), Size::new(0, 0)),
		}
	}

	pub fn new_window_node(shell_handle: WLRXdgV6ShellSurfaceHandle, parent_node_index: usize) -> Self {
		LayoutNode::new(LayoutNodeType::Window(shell_handle), parent_node_index)
	}

	pub fn new_container_node(container_node_data: ContainerData, parent_node_index: usize) -> Self {
		LayoutNode::new(LayoutNodeType::Container(container_node_data), parent_node_index)
	}

	pub fn is_node_containing_shell_handle(&self, shell_handle: &WLRXdgV6ShellSurfaceHandle) -> bool {
		match &self.node_type {
			LayoutNodeType::Window(node_shell_handle) => *node_shell_handle == *shell_handle,
			_ => false,
		}
	}

	pub fn update_area(&mut self, parent_area: &Area, parent_axis: &ContainerAxis, siblings_weight_sum: f32, origin_offset: i32) {
		// ? Change area based on parent and siblings changes
		match parent_axis {
			ContainerAxis::Horizontal => {
				let area_width =
					((self.weight / siblings_weight_sum) * parent_area.size.width as f32).ceil() as i32;
				let area_height = parent_area.size.height;
				let new_area = Area::new(
					Origin::new(origin_offset, parent_area.origin.y),
					Size::new(area_width, area_height),
				);
				self.area = new_area;
			}
			ContainerAxis::Vertical => {
				let area_width = parent_area.size.width;
				let area_height =
					((self.weight / siblings_weight_sum) * parent_area.size.height as f32).ceil() as i32;
				let new_area = Area::new(
					Origin::new(parent_area.origin.x, origin_offset),
					Size::new(area_width, area_height),
				);
				self.area = new_area;
			}
		}

		// ? Apply new size to the shell
		if let LayoutNodeType::Window(shell_handle) = &self.node_type {
			shell_handle.run(|shell| {
				if let Some(WLRXdgV6ShellState::TopLevel(ref mut xdg_top_level)) = shell.state() {
					xdg_top_level.set_size(self.area.size.width as u32, self.area.size.height as u32);
				}
			}).unwrap();
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

	pub fn update_area(&mut self, area: Area) {
		if let Some(ref mut root_node) = self.nodes[0] {
			root_node.area = area;
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
			Some(root_node.area)
		} else {
			None
		}
	}

	pub fn windows_data(&self) -> Vec<WindowData> {
		self
			.nodes
			.iter()
			.filter_map(|element| {
				if let Some(LayoutNode { node_type, area, .. }) = element {
					if let LayoutNodeType::Window(shell_handle) = node_type {
						return Some(WindowData::new(shell_handle.clone(), area.clone()));
					}
				}
				None
			}).collect()
	}

	pub fn parent_node_index_of(&self, node_index: usize) -> Option<usize> {
		if let Some(Some(node)) = self.nodes.get(node_index) {
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

	pub fn get_node_area(&self, node_index: usize) -> Option<Area> {
		if let Some(node) = &self.nodes[node_index] {
			return Some(node.area);
		}
		None
	}

	pub fn rebalance_container(&mut self, node_index: usize) {
		let mut node_indices_to_rebalance = vec![node_index];
		while let Some(node_index) = node_indices_to_rebalance.pop() {
			if let Some(container_data) = self.get_container_data_of_node(node_index) {
				if let Some(container_area) = self.get_node_area(node_index) {
					let children_weight_sum: f32 = container_data.get_children_weight_sum(&self.nodes);
					let mut origin_offset: i32 = 0;
					for child_index in container_data.children_indices {
						if let Some(Some(child_node)) = self.nodes.iter_mut().nth(child_index) {
							child_node.update_area(&container_area, &container_data.axis, children_weight_sum, origin_offset);

							match container_data.axis {
								ContainerAxis::Horizontal => {
									origin_offset += child_node.area.size.width;
								}
								ContainerAxis::Vertical => {
									origin_offset += child_node.area.size.height;
								}
							}

							if let LayoutNodeType::Container(_) = child_node.node_type {
								node_indices_to_rebalance.push(child_index);
							}
						}
					}
				}
			}
		}
	}

	pub fn add_child_to_container_of_active_node(&mut self, node_index: usize) {
		let parent_node_index = self.parent_node_index_of_active_node();
		// ? Set parent to child
		if let Some(ref mut node) = self.nodes[node_index] {
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

	pub fn index_of_node_containing_shell(&self, shell_handle: &WLRXdgV6ShellSurfaceHandle) -> Option<usize> {
		self.nodes.iter().position(|element| match *element {
			Some(ref node) => node.is_node_containing_shell_handle(shell_handle),
			None => false,
		})
	}

	pub fn set_activated(&mut self, node_index: usize) {
		self.active_node_index = node_index;
		if let Some(Some(node)) = self.nodes.get(node_index) {
			// TODO: Handle 'set activated' on Container node
			if let LayoutNodeType::Window(ref shell_handle) = node.node_type {
				shell_handle.run(|shell| {
					if let Some(WLRXdgV6ShellState::TopLevel(ref mut xdg_top_level)) = shell.state() {
						xdg_top_level.set_activated(true);
					}
				}).unwrap();
			}
		}
	}

	pub fn add_window(&mut self, shell_handle: WLRXdgV6ShellSurfaceHandle) {
		let node_index = self.add_node(LayoutNode::new_window_node(shell_handle, 0));
		self.add_child_to_container_of_active_node(node_index);
		self.nb_windows += 1;
		self.set_activated(node_index);
	}

	pub fn find_fallback_node_index(&self, node_index: usize) -> Option<usize> {
		let mut parent_node_index = 0;
		if let Some(Some(node)) = self.nodes.get(self.active_node_index) {
			parent_node_index = node.parent_node_index;
		}

		// ? Use left sibling or right sibling as fallback
		if let Some(Some(parent_node)) = &self.nodes.get(parent_node_index) {
			if let LayoutNodeType::Container(container_data) = &parent_node.node_type {
				let index_of_child = container_data.index_of(node_index).unwrap();
				if index_of_child > 0 {
					// ? Left sibling
					if let Some(sibling_index) = container_data.children_indices.get(index_of_child - 1) {
						return Some(*sibling_index);
					}
					// ? Right sibling
					if let Some(sibling_index) = container_data.children_indices.get(index_of_child + 1) {
						return Some(*sibling_index);
					}
				}
			}

			// ? If parent has no other child and parent is root_node
			if parent_node_index == 0 {
				return None;
			}
		}

		// ? Check for a fallback for the parent
		self.find_fallback_node_index(parent_node_index)
	}

	pub fn get_fallback_shell_handle(&self) -> Option<WLRXdgV6ShellSurfaceHandle> {
		if let Some(index_of_fallback) = self.find_fallback_node_index(self.active_node_index) {
			if let Some(Some(node)) = self.nodes.get(index_of_fallback) {
				if let LayoutNodeType::Window(shell_handle) = &node.node_type {
					let shell_handle_clone = shell_handle.clone();
					return Some(shell_handle_clone);
				}
			}
		}

		None
	}

	pub fn remove_node(&mut self, node_index: usize) {
		// ? Can't remove root node
		if node_index == 0 { return; }

		// ? Iterative and accending remove
		let mut indices_of_nodes_to_remove = vec![node_index];
		let mut index_of_container_to_rebalance = 0;
		while let Some(index_of_node_to_remove) = indices_of_nodes_to_remove.pop() {
			// ? Remove from parent
			if let Some(parent_node_index) = self.parent_node_index_of(index_of_node_to_remove) {
				if let Some(parent_node) = &mut self.nodes[parent_node_index] {
					if let LayoutNodeType::Container(ref mut container_node_data) = parent_node.node_type {
						if let Some(index_of_child) = container_node_data.index_of(index_of_node_to_remove) {
							container_node_data.children_indices.remove(index_of_child);

							// ? Remove parent if empty
							if container_node_data.children_indices.len() == 0 && parent_node_index != 0 {
								indices_of_nodes_to_remove.push(parent_node_index);
							} else {
								index_of_container_to_rebalance = parent_node_index;
							}
						}
					}
				}
			}

			// ? Remove the node
			self.nodes[index_of_node_to_remove] = None;
		}

		// ? Rebalance the original parent node
		self.rebalance_container(index_of_container_to_rebalance);
	}

	pub fn remove_window(&mut self, shell_handle: &WLRXdgV6ShellSurfaceHandle) -> Option<WLRXdgV6ShellSurfaceHandle> {
		if let Some(node_index) = self.index_of_node_containing_shell(shell_handle) {
			if let Some(fallback_node_index) = self.find_fallback_node_index(node_index) {
				self.remove_node(node_index);

				if let Some(node) = &self.nodes[fallback_node_index] {
					if let LayoutNodeType::Window(fallback_shell_handle) = &node.node_type {
						return Some(fallback_shell_handle.clone());
					}
				}
			}
		}

		None
	}

	pub fn contains_window(&self, shell_handle: &WLRXdgV6ShellSurfaceHandle) -> bool {
		for element in &self.nodes {
			if let Some(node) = element {
				if let LayoutNodeType::Window(node_shell_handle) = &node.node_type {
					if *node_shell_handle == *shell_handle {
						return true;
					}
				}
			}
		}
		false
	}
}
