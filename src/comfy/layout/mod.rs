use std::collections::{LinkedList, HashSet, HashMap};

use wlroots::{Area, IntersectionResult, Origin, Size, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle, XdgV6ShellState as WLRXdgV6ShellState};

use compositor::window::Window;

/*
..####....####...##..##...####...######...####...##..##..######...####..
.##..##..##..##..###.##..##........##....##..##..###.##....##....##.....
.##......##..##..##.###...####.....##....######..##.###....##.....####..
.##..##..##..##..##..##......##....##....##..##..##..##....##........##.
..####....####...##..##...####.....##....##..##..##..##....##.....####..
........................................................................
*/

type NodeIndex = usize;
const INDEX_OF_ROOT: NodeIndex = 0;

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
pub enum LayoutAxis {
	Vertical,
	Horizontal,
}

#[derive(Clone, Eq, PartialEq)]
pub enum LayoutDirection {
	Up,
	Down,
	Left,
	Right,
}
impl LayoutDirection {
	pub fn get_axis(&self) -> LayoutAxis {
		match self {
			LayoutDirection::Up | LayoutDirection::Down => LayoutAxis::Vertical,
			LayoutDirection::Left | LayoutDirection::Right => LayoutAxis::Horizontal,
		}
	}
}

pub enum RelativePosition {
	After,
	Before
}

#[derive(Clone, Eq, PartialEq)]
pub struct ContainerData {
	pub children_indices: Vec<NodeIndex>,
	pub axis: LayoutAxis,
}

impl ContainerData {

	/// Creates a new instance of ContainerData.
	fn new(children_indices: Vec<NodeIndex>, axis: LayoutAxis) -> Self {
		ContainerData { children_indices, axis }
	}

	/// Returns the number of child contained inside the container.
	fn len(&self) -> usize {
		self.children_indices.len()
	}

	fn set_axis(&mut self, new_axis: LayoutAxis) {
		self.axis = new_axis;
	}

	/// Adds the provided index as a child of the container.
	fn add_child_index(&mut self, child_index: NodeIndex) {
		self.children_indices.push(child_index);
	}

	fn add_child_index_after(&mut self, index_to_add: NodeIndex, value_of_target: NodeIndex) {
		if let Some(index_of_target) = self.index_of(value_of_target) {
			self.children_indices.insert(index_of_target + 1, index_to_add);
		}
	}

	fn add_child_index_before(&mut self, index_to_add: NodeIndex, value_of_target: NodeIndex) {
		if let Some(index_of_target) = self.index_of(value_of_target) {
			self.children_indices.insert(index_of_target, index_to_add);
		}
	}

	fn _get_children_weight_sum(&self, nodes: &Vec<Option<LayoutNode>>) -> f32 {
		self
			.children_indices
			.iter()
			.map(|index| match &nodes[*index] {
				Some(layout_node) => layout_node.weight,
				None => 0.0,
			}).sum()
	}

	/// If `child_value` exists as a child of the container, returns it's index in the container.
	fn index_of(&self, child_value: NodeIndex) -> Option<NodeIndex> {
		self.children_indices.iter().position(|element| *element == child_value)
	}

	/// If `child_value` exists as a child of the container, removes it from the container.
	fn remove(&mut self, child_value: NodeIndex) {
		if let Some(index_of_child) = self.index_of(child_value) {
			self.children_indices.remove(index_of_child);
		}
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

#[derive(Clone)]
pub enum LayoutNodeType {
	Container(ContainerData),
	Window(Window),
}

#[derive(Clone)]
pub struct LayoutNode {
	node_type: LayoutNodeType,
	parent_node_index: NodeIndex,
	weight: f32,
	area: Area,
}

impl LayoutNode {
	/// Create a new node with the itself as it's parent.
	pub fn new_root_node(area: Area) -> Self {
		let mut node = LayoutNode::new_container_node(ContainerData::new(vec![], LayoutAxis::Horizontal), 0);
		node.area = area;
		node
	}

	/// Create a new node with the provided node type and binds it to the provided parent node index.
	pub fn new(layout_node_type: LayoutNodeType, parent_node_index: NodeIndex) -> Self {
		LayoutNode {
			node_type: layout_node_type,
			parent_node_index,
			weight: 1.0,
			area: Area::new(Origin::new(0, 0), Size::new(0, 0)),
		}
	}

	/// Create a new node that contains a window and binds it to the provided parent node index.
	pub fn new_window_node(window: Window, parent_node_index: NodeIndex) -> Self {
		LayoutNode::new(LayoutNodeType::Window(window), parent_node_index)
	}

	/// Create a new container node and binds it to the provided parent node index.
	pub fn new_container_node(container_node_data: ContainerData, parent_node_index: NodeIndex) -> Self {
		LayoutNode::new(LayoutNodeType::Container(container_node_data), parent_node_index)
	}

	pub fn is_window_node(&self) -> bool {
		match self.node_type {
			LayoutNodeType::Window(_) => true,
			_ => false,
		}
	}

	/// Returns true if the node contains the provided xdg surface shell handle.
	pub fn is_containing_shell_handle(&self, shell_handle: &WLRXdgV6ShellSurfaceHandle) -> bool {
		match &self.node_type {
			LayoutNodeType::Window(Window {shell_handle: window_shell_handle, ..}) => *window_shell_handle == *shell_handle,
			_ => false,
		}
	}

	/// Rebalances the node given it's parent area and axis.
	/// The dimensions rebalancing will be calculated given the siblings weight sum and the position from the origin_offeset.
	/// Returns true if the area changed.
	pub fn rebalance_area(
		&mut self,
		parent_area: &Area,
		parent_axis: &LayoutAxis,
		siblings_weight_sum: f32,
		previous_siblings_offset: i32,
	) -> bool {
		// ? Change area based on parent and siblings changes
		let new_area = match parent_axis {
			LayoutAxis::Horizontal => {
				let area_width = ((self.weight / siblings_weight_sum) * parent_area.size.width as f32).ceil() as i32;
				let area_height = parent_area.size.height;
				Area::new(
					Origin::new(parent_area.origin.x + previous_siblings_offset, parent_area.origin.y),
					Size::new(area_width, area_height),
				)
			}
			LayoutAxis::Vertical => {
				let area_width = parent_area.size.width;
				let area_height = ((self.weight / siblings_weight_sum) * parent_area.size.height as f32).ceil() as i32;
				Area::new(
					Origin::new(parent_area.origin.x, parent_area.origin.y + previous_siblings_offset),
					Size::new(area_width, area_height),
				)
			}
		};

		let area_changed = if self.area != new_area {
			self.area = new_area;
			true
		} else {
			false
		};

		area_changed
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
	pub available_places: Vec<NodeIndex>,
	pub nodes: Vec<Option<LayoutNode>>,
	pub ordered_window_node_indices: Vec<NodeIndex>,
	pub active_node_index: NodeIndex,
}

impl Layout {
	/// Create a new layout that occupates the space of the provided area
	pub fn new(output_area: Area) -> Self {
		let root_node = LayoutNode::new_root_node(output_area);
		Layout {
			available_places: vec![],
			nodes: vec![Some(root_node)],
			ordered_window_node_indices: vec![],
			active_node_index: INDEX_OF_ROOT,
		}
	}

	/// Returns a vector containing the windows indices of a windows contained in this layout.
	pub fn get_windows(&self) -> Vec<Window> {
		let mut windows = vec![];
		for window_node_index in self.ordered_window_node_indices.iter() {
			if let Some(Some(node)) = self.nodes.get(*window_node_index) {
				if let LayoutNodeType::Window(ref window) = node.node_type {
					windows.push(window.clone());
				}
			}
		}
		windows
	}

	/// Updates the render area of the layout.
	/// Does not rebalances the tree.
	pub fn update_area(&mut self, area: Area) {
		if let Some(ref mut root_node) = self.nodes[INDEX_OF_ROOT] {
			root_node.area = area;
		}
	}

	/// Updates the area of the layout then rebalances the tree from the root.
	pub fn update_area_and_rebalance(&mut self, area: Area) {
		self.update_area(area);
		self.rebalance_root();
	}

	/// Returns the render box of the layout.
	pub fn area(&self) -> Option<Area> {
		if let Some(ref root_node) = self.nodes[INDEX_OF_ROOT] {
			Some(root_node.area)
		} else {
			None
		}
	}

	/// Returns the shell handle of the active node if any.
	pub fn get_active_shell_handle(&self) -> Option<WLRXdgV6ShellSurfaceHandle> {
		if let Some(Some(node)) = self.nodes.get(self.active_node_index) {
			match node.node_type {
				LayoutNodeType::Window(Window {ref shell_handle, ..}) => return Some(shell_handle.clone()),
				_ => {}
			}
		}
		None
	}

	/// Returns the index of the parent of the node associated with the provided node index
	fn get_parent_node_index_of(&self, node_index: NodeIndex) -> Option<NodeIndex> {
		if let Some(Some(node)) = self.nodes.get(node_index) {
			Some(node.parent_node_index)
		} else {
			None
		}
	}

	/// Returns the node index of the parent of the active node.
	fn get_parent_node_index_of_active_node(&self) -> Option<NodeIndex> {
		self.get_parent_node_index_of(self.active_node_index)
	}

	/// Adds a new node inside layout's list of nodes. If there is holes in the list (available places), one of them will be used.
	fn add_node_to_list(&mut self, layout_node: LayoutNode) -> NodeIndex {
		if let Some(available_index) = self.available_places.pop() {
			self.nodes[available_index] = Some(layout_node);
			available_index
		} else {
			self.nodes.push(Some(layout_node));
			self.nodes.len() - 1
		}
	}

	/// Returns the area of the node given it's node index
	fn get_node_area(&self, node_index: NodeIndex) -> Option<Area> {
		if let Some(Some(node)) = self.nodes.get(node_index) {
			return Some(node.area);
		}
		None
	}

	/// Find the index of the previous sibling of the provided node index.
	/// The previous sibling is the closest accessible node of the tree (which is not an ancestor) from a specific node, going from right to left.
	fn get_index_of_previous_sibling(&self, node_index: NodeIndex, only_direct_sibling: bool) -> Option<NodeIndex> {
		// ? Can't find previous sibling of root because root as no siblings
		if node_index != INDEX_OF_ROOT {
			return None;
		}

		// ? Recursive search
		let mut index_of_previous_sibling_option = None;
		if let Some(parent_index) = self.get_parent_node_index_of(node_index) {
			if let Some(Some(parent_node)) = self.nodes.get(parent_index) {
				match parent_node.node_type {
					LayoutNodeType::Container(ref container_data) => {

						// ? Find the index of the child which is equal to the node_index
						let child_position = container_data.children_indices
							.iter()
							.position(|&value| value == node_index)
							.unwrap();
						let has_no_previous_sibling = child_position == 0;

						// ? If there is no previous sibling in the container, find the previous sibling of the parent.
						index_of_previous_sibling_option = if has_no_previous_sibling {
							self.get_index_of_previous_sibling(parent_index, only_direct_sibling)
						} else {
							Some(container_data.children_indices[child_position - 1])
						}
					},
					_ => error!("While searching for the index of the previous sibling of a node, get_parent_node_index_of returned a window node which should never happen!"),
				}
			}
		}
		index_of_previous_sibling_option
	}

	/// Returns the area of the previous sibling if there is one for the provided node index.
	fn get_area_of_previous_sibling(&self, node_index: NodeIndex, only_direct_sibling: bool) -> Option<Area> {
		let mut area_of_previous_sibling_option = None;
		if let Some(index_of_previous_sibling) = self.get_index_of_previous_sibling(node_index, only_direct_sibling) {
			if let Some(Some(previous_sibling_node)) = self.nodes.get(index_of_previous_sibling) {
				area_of_previous_sibling_option = Some(previous_sibling_node.area);
			}
		}
		area_of_previous_sibling_option
	}

	/// Returns the area of the parent node of the provided node index if any.
	fn get_area_of_parent_node(&self, node_index: NodeIndex) -> Option<Area> {
		let mut parent_area = None;
		if let Some(parent_index) = self.get_parent_node_index_of(node_index) {
			if let Some(Some(parent_node)) = self.nodes.get(parent_index) {
				parent_area = Some(parent_node.area);
			}
		}
		parent_area
	}

	/// Given a container node index, returns a clone of it's container data.
	pub fn get_container_data_of_node(&self, container_node_index: usize) -> Option<ContainerData> {
		if let Some(parent_node) = &self.nodes[container_node_index] {
			if let LayoutNodeType::Container(container_node_data) = &parent_node.node_type {
				return Some(container_node_data.clone());
			}
		}
		None
	}

	/// Returns the weight sum of all the direct children of the provided node.
	fn get_direct_children_weight_sum(&self, node_index: NodeIndex) -> Option<f32> {
		if let Some(Some(node)) = self.nodes.get(node_index).clone() {
			match node.node_type {
				LayoutNodeType::Container(ref container_data) => {
					return Some(
						container_data.children_indices
							.iter()
							.map(|index| match self.nodes[*index] {
								Some(ref layout_node) => layout_node.weight,
								None => 0.0,
							}).sum()
					);
				},
				_ => {}
			}
		}
		None
	}

	/// Recalculates the position and dimensions of each node from the root.
	pub fn rebalance_root(&mut self) {
		self.rebalance_subtree(INDEX_OF_ROOT)
	}

	/// Recalculates the position and dimensions of each node from the provided node index.
	fn rebalance_subtree(&mut self, subtree_root_node: NodeIndex) {
		let mut indices_of_nodes_to_rebalance = vec![subtree_root_node];
		while let Some(index_of_node_to_rebalance) = indices_of_nodes_to_rebalance.pop() {
			if let Some(container_data) = self.get_container_data_of_node(index_of_node_to_rebalance) {
				if let Some(container_area) = self.get_node_area(index_of_node_to_rebalance) {
					let children_weight_sum: f32 = self.get_direct_children_weight_sum(index_of_node_to_rebalance).unwrap();
					let mut origin_offset: i32 = 0;
					for child_index in container_data.children_indices {
						if let Some(Some(child_node)) = self.nodes.get_mut(child_index) {
							child_node.rebalance_area(
								&container_area,
								&container_data.axis,
								children_weight_sum,
								origin_offset,
							);

							match container_data.axis {
								LayoutAxis::Horizontal => {
									origin_offset += child_node.area.size.width;
								}
								LayoutAxis::Vertical => {
									origin_offset += child_node.area.size.height;
								}
							}

							match child_node.node_type {
								LayoutNodeType::Container(_) => indices_of_nodes_to_rebalance.push(child_index),
								LayoutNodeType::Window(ref mut window) => window.resize(child_node.area)
							}
						}
					}
				}
			}
		}
	}

	/// Move the node associated to the provided node index as the last child of a provided target.
	fn move_index_under(&mut self, node_index: NodeIndex, index_of_target: NodeIndex) -> Option<NodeIndex> {

		// ? Remove child from current parent
		if let Some(parent_node_index) = self.get_parent_node_index_of(node_index) {
			if let Some(Some(parent_node)) = self.nodes.get_mut(parent_node_index) {
				if let LayoutNodeType::Container(ref mut parent_container_data) = parent_node.node_type {
					parent_container_data.remove(node_index);
				}
			}
		}

		// ? Set parent to child
		if let Some(Some(node)) = self.nodes.get_mut(node_index) {
			node.parent_node_index = index_of_target;
		}

		// ? Set child to parent
		if let Some(Some(parent_node)) = self.nodes.get_mut(index_of_target) {
			if let LayoutNodeType::Container(ref mut container_node_data) = parent_node.node_type {
				container_node_data.add_child_index(node_index);
				return Some(index_of_target);
			}
		}

		None
	}

	/// Moves the nodes associated with the provided node indices next to the target.
	fn move_indices_next_to(&mut self, indices_to_move: Vec<NodeIndex>, index_of_target: NodeIndex, relative_position: &RelativePosition) {
		for index_to_move in indices_to_move.iter().rev() {
			self.move_index_next_to(*index_to_move, index_of_target, relative_position);
		}
	}

	/// Moves the nodes associated with the direct children of the provided `parent_node_index` next to the target.
	fn move_direct_children_next_to(&mut self, parent_node_index: NodeIndex, index_of_target: NodeIndex, relative_position: &RelativePosition) {
		let direct_children_indices = self.get_direct_children_indices_of(parent_node_index);
		self.move_indices_next_to(direct_children_indices, index_of_target, relative_position);
	}

	/// Moves a node in the layout next to another.
	fn move_index_next_to(&mut self, index_of_node_to_move: NodeIndex, index_of_target: NodeIndex, relative_position: &RelativePosition) -> Option<NodeIndex> {

		// ? Get parent node index, otherwise print an error (parent of target has to exist or target isn't a valid node)
		if let Some(index_of_parent_of_target) = self.get_parent_node_index_of(index_of_target) {

			// ? Remove node to add from it's parent
			if let Some(parent_index_of_node_to_add) = self.get_parent_node_index_of(index_of_node_to_move) {
				if let Some(Some(parent_of_node_to_add)) = self.nodes.get_mut(parent_index_of_node_to_add) {
					if let LayoutNodeType::Container(ref mut parent_container_data) = parent_of_node_to_add.node_type {
						parent_container_data.remove(index_of_node_to_move);
					}
				}
			}

			// ? Set parent to child
			if let Some(Some(node)) = self.nodes.get_mut(index_of_node_to_move) {
				node.parent_node_index = index_of_parent_of_target;
			}

			// ? Set child to parent
			if let Some(Some(parent_node)) = self.nodes.get_mut(index_of_parent_of_target) {
				if let LayoutNodeType::Container(ref mut parent_container_data) = parent_node.node_type {
					match relative_position {
						RelativePosition::After => parent_container_data.add_child_index_after(index_of_node_to_move, index_of_target),
						RelativePosition::Before => parent_container_data.add_child_index_before(index_of_node_to_move, index_of_target),
					}
					return Some(index_of_parent_of_target);
				}
			}
		} else {
			error!("INVALID TARGET NODE: Tried to add index '{}' next to '{}', but target has no parent.", index_of_node_to_move, index_of_target);
		}

		None
	}

	fn get_axis_of(&self, node_index: NodeIndex) -> Option<LayoutAxis> {
		if let Some(Some(node)) = self.nodes.get(node_index) {
			if let LayoutNodeType::Container(ref container_data) = node.node_type {
				return Some(container_data.axis.clone());
			}
		}
		None
	}

	/// Adds a new container to the layout.
	fn add_new_container(&mut self, axis: LayoutAxis, parent_node_index: NodeIndex) -> NodeIndex {
		let container_node = LayoutNode::new_container_node(
			ContainerData::new(vec![], axis),
			parent_node_index
		);

		self.add_node_to_list(container_node)
	}

	/// Binds the provided index as a neighbor of the active node.
	/// Returns the index of the parent node if if was successfully added.
	fn move_index_relative_to_active_node(&mut self, index_of_node_to_add: NodeIndex, direction: &LayoutDirection) -> Option<NodeIndex> {
		let active_node_is_root = self.active_node_index == INDEX_OF_ROOT;
		let parent_node_index = if active_node_is_root {
			0
		} else {
			self.get_parent_node_index_of_active_node().unwrap_or_else(|| INDEX_OF_ROOT)
		};

		let axis_of_parent = self.get_axis_of(parent_node_index).unwrap();
		let nb_children_of_parent = self.get_direct_children_indices_of(parent_node_index).len();
		match (nb_children_of_parent, direction.get_axis() == axis_of_parent) {

			// ? Parent has only 0 or 1 children, we don't care about the axis since where going to change it
			(0 ... 1, _) => {
				if active_node_is_root {

					// ? Active node is root, just add new index under it
					self.move_index_under_root(index_of_node_to_add);
				} else {

					// ? Move the new node before or after the active node (depending on the given direction)
					let active_node_index = self.active_node_index;
					match direction {
						LayoutDirection::Left | LayoutDirection::Up =>
							self.move_index_next_to(index_of_node_to_add, active_node_index, &RelativePosition::Before),
						LayoutDirection::Right | LayoutDirection::Down =>
							self.move_index_next_to(index_of_node_to_add, active_node_index, &RelativePosition::After),
					};
				}

				// ? Change the axis of the parent node
				if let Some(Some(parent_node)) = self.nodes.get_mut(parent_node_index) {
					if let LayoutNodeType::Container(ref mut container_data) = parent_node.node_type {
						container_data.set_axis(direction.get_axis());
					}
				}

				return Some(parent_node_index);
			},

			// ? Parent has more than 1 child, but is on the same axis
			(_, true) => {
				let active_node_index = self.active_node_index;
				match direction {
					LayoutDirection::Left | LayoutDirection::Up =>
						self.move_index_next_to(index_of_node_to_add, active_node_index, &RelativePosition::Before),
					LayoutDirection::Right | LayoutDirection::Down =>
						self.move_index_next_to(index_of_node_to_add, active_node_index, &RelativePosition::After),
				};
				return Some(parent_node_index);
			},

			// ? Parent has more than 1 child, but is on a different axis
			(_, false) => {
				let active_node_index = self.active_node_index;

				// ? Creates new container node to which we will add the active node and the new node
				let new_container_index = self.add_new_container(direction.get_axis(), parent_node_index);

				// ? Move the container next to the active node
				self.move_index_next_to(new_container_index, active_node_index, &RelativePosition::After);

				// ? Move the active node under the new container
				self.move_index_under(active_node_index, new_container_index);

				// ? Move the new node before or after the active node index (depending on the given direction)
				let relative_position_where_to_move = match direction {
					LayoutDirection::Left | LayoutDirection::Up => RelativePosition::Before,
					LayoutDirection::Right | LayoutDirection::Down => RelativePosition::After,
				};
				self.move_index_next_to(index_of_node_to_add, active_node_index, &relative_position_where_to_move);
				return Some(parent_node_index);
			}
			_ => {
				// TODO: ERROR
			}
		}

		None
	}

	/// Returns the index of the node containing the provided xdg shell surface handle.
	pub fn index_of_node_containing_shell_handle(&self, shell_handle: &WLRXdgV6ShellSurfaceHandle) -> Option<NodeIndex> {
		self.nodes.iter().position(|element| match *element {
			Some(ref node) => node.is_containing_shell_handle(shell_handle),
			None => false,
		})
	}

	/// Sets the provided node index as the last activated node of the layout.
	/// The last activated node will gain focus when the layout gains focus.
	fn set_as_last_activated(&mut self, node_index: NodeIndex) {
		self.active_node_index = node_index;
	}

	/// Moves the provided index under the root node
	fn move_index_under_root(&mut self, node_index: NodeIndex) -> Option<NodeIndex> {
		self.move_index_under(node_index, INDEX_OF_ROOT)
	}

	/// Adds a window in the layout given it's associated xdg shell surface handle.
	/// The containing node will be a neighbor of the currently activated node if any.
	/// Otherwise, it will be added as a child of the root.
	pub fn add_window(&mut self, window: Window, direction: &LayoutDirection, set_as_last_activated: bool) -> Option<NodeIndex> {
		let node_index = self.add_node_to_list(LayoutNode::new_window_node(window, INDEX_OF_ROOT));
		let index_of_parent_option = if self.active_node_index == INDEX_OF_ROOT {
			self.move_index_under_root(node_index)
		} else {
			self.move_index_relative_to_active_node(node_index, direction)
		};

		// ? If a parent was return, we set the window node as activated, else we undo the insertion.
		if index_of_parent_option.is_none() {
			self.remove_node(node_index);
		} else {
			if set_as_last_activated {
				self.set_as_last_activated(node_index);
			}
			self.ordered_window_node_indices.push(node_index);
			self.print_subtree_to_console(INDEX_OF_ROOT);
		}

		index_of_parent_option
	}

	/// Adds a window in the layout given it's associated xdg shell surface handle.
	/// The containing node will be a neighbor of the currently activated node if any.
	/// Otherwise, it will be added as a child of the root.
	/// Ends with a rebalancing of the layout tree from the top parent.
	pub fn add_window_and_rebalance(&mut self, window: Window, direction: &LayoutDirection, set_as_last_activated: bool) {
		let modified_parent_index = self.add_window(window, direction, set_as_last_activated).unwrap();
		self.rebalance_subtree(modified_parent_index)
	}

	/// If the layout contains a window associated with the provided xdg shell surface handle, we remove it from the layout.
	/// Returns the node index of the root of the subtree that needs to be rebalanced.
	pub fn remove_window_from_shell_handle(&mut self, shell_handle: &WLRXdgV6ShellSurfaceHandle) -> Option<usize> {
		if let Some(node_index) = self.index_of_node_containing_shell_handle(shell_handle) {
			self.remove_node(node_index)
		} else {
			None
		}
	}

	/// If the layout contains a window associated with the provided xdg shell surface handle, we remove it from the layout.
	pub fn remove_window_from_shell_handle_and_rebalance(&mut self, shell_handle: &WLRXdgV6ShellSurfaceHandle) {
		if let Some(node_index_to_rebalance) = self.remove_window_from_shell_handle(shell_handle) {
			self.rebalance_subtree(node_index_to_rebalance);
		}
	}

	/// Return the index of the node that would be the active one in the case of the active node being deleted.
	pub fn find_fallback_node_index(&self, node_index: NodeIndex) -> NodeIndex {
		let mut parent_node_index = 0;
		if let Some(index) = self.get_parent_node_index_of(self.active_node_index) {
			parent_node_index = index;
		}

		// ? Use left sibling or right sibling as fallback
		if let Some(Some(parent_node)) = &self.nodes.get(parent_node_index) {
			if let LayoutNodeType::Container(container_data) = &parent_node.node_type {
				let index_of_child = container_data.index_of(node_index).unwrap();
				if index_of_child > 0 {

					// ? Left sibling
					if let Some(previous_sibling_index) = container_data.children_indices.get(index_of_child - 1) {
						return *previous_sibling_index;
					}

					// ? Right sibling
					if index_of_child < container_data.children_indices.len() - 1 {
						if let Some(next_sibling_index) = container_data.children_indices.get(index_of_child + 1) {
							return *next_sibling_index;
						}
					}
				}
			}
		}

		// ? If parent has no other child and parent is root_node
		if parent_node_index == 0 {
			return 0;
		}

		// ? Check for a fallback for the parent
		self.find_fallback_node_index(parent_node_index)
	}

	/// Iteratively removes holes in the list of nodes from the back to the front.
	/// Stops when it finds a value that is not a None.
	fn clear_node_list_trailing_holes(&mut self) {

		// ? Backward search until a `Some` value is found.
		for i in (1..self.nodes.len()).rev() {
			if self.nodes[i].is_some() {
				break;
			}
			self.nodes.remove(i);
			if let Some(index_of_available_place) = self.available_places.iter().position(|&value| value == i) {
				self.available_places.remove(index_of_available_place);
			}
		}
	}

	/// Removes the provided node index from the list of nodes.
	/// If the node is the currently active one, assign the active to the fallback node.
	/// Also removes trailing holes dynamically if the removed node is the last one in the list.
	fn remove_node_from_list(&mut self, node_index: NodeIndex) {

		// ? If the node is the active one, set active to fallback.
		if node_index == self.active_node_index {
			self.active_node_index = self.find_fallback_node_index(node_index);
		}

		// ? If the node index to remove is the last of the list, clear trailing holes.
		self.nodes[node_index] = None;

		if let Some(index_in_ordered_indices) = self.ordered_window_node_indices.iter().position(|&value| value == node_index) {
			self.ordered_window_node_indices.remove(index_in_ordered_indices);
		}

		// ? If the removed node is the last of the list, clear trailing holes. Otherwise create one.
		if node_index == self.nodes.len() - 1 {
			self.clear_node_list_trailing_holes();
		} else {
			self.available_places.push(node_index);
		}
	}

	/// Removes all the children of a subtree from the layout.
	/// Since we want to remove all children, it doesn't unassign subchildren from parents except for the subtree root.
	pub fn remove_all_children_of(&mut self, subtree_root_index: NodeIndex) {
		let indices_of_nodes_to_remove = self.get_indices_of_subtree(subtree_root_index, false, true);

		// ? Remove all children from the subtree iteratively
		for index_of_node_to_remove in indices_of_nodes_to_remove.iter() {
			self.remove_node_from_list(*index_of_node_to_remove);
		}

		// ? If the subtree root is a container, clear the children
		if let Some(Some(node)) = self.nodes.get_mut(subtree_root_index) {
			if let LayoutNodeType::Container(ref mut container_data) = node.node_type {
				container_data.children_indices.clear();
			}
		}
	}

	/// Returns all the node indices of the subtree from the provided root.
	/// If `only_leafs` is true, skips all container nodes.
	pub fn get_indices_of_subtree(&self, subtree_root_index: NodeIndex, only_leafs: bool, exclude_root: bool) -> Vec<NodeIndex> {
		let mut indices_of_subtree = vec![];

		// ? Iteratively add all node indices to the indices_of_subtree
		let mut indices_to_check = vec![subtree_root_index];
		while let Some(node_index) = indices_to_check.pop() {
			if let Some(ref node) = self.nodes[node_index] {
				match node.node_type {
					LayoutNodeType::Container(ref container) => {
						if !only_leafs {
							indices_of_subtree.push(node_index);
						}

						// ? Add all child indices to the indices to check
						for i in container.children_indices.iter() {
							indices_to_check.push(*i);
						}
					}
					_ => {
						indices_of_subtree.push(node_index);
					}
				}
			}
		}

		if exclude_root {
			indices_of_subtree.remove(0);
		}

		indices_of_subtree
	}

	/// Returns a vector containing all the indices of the direct children of a node.
	fn get_direct_children_indices_of(&self, node_index: NodeIndex) -> Vec<NodeIndex> {
		let mut direct_children_of = Vec::new();
		if let Some(Some(node)) = self.nodes.get(node_index) {
			match node.node_type {
				LayoutNodeType::Container(ref container_data) => {
					for child_value in container_data.children_indices.iter() {
						direct_children_of.push(*child_value)
					}
				},
				_ => {}
			}
		}
		direct_children_of
	}

	/// Removes the node associated with the provided node index from the layout.
	/// Will remove all it's children and it's parent if it has less than 2 children after removal of the child.
	/// Returns the index of the top node which needs to be rebalanced.
	pub fn remove_node(&mut self, node_index: NodeIndex) -> Option<NodeIndex> {

		// ? Can't remove root node
		if node_index == 0 {
			error!("Tried to remove root index of layout!");
			return None;
		}

		// ? Validate the provided node index (can't remove invalid node)
		if !(0 < node_index && node_index < self.nodes.len()) || self.nodes[node_index].is_none() {
			error!("Tried to remove invalid node index '{}' from layout!", node_index);
			return None;
		}

		let mut index_of_container_to_rebalance = None;

		// ? keep hook on parent index for further detach
		let parent_node_index = self.get_parent_node_index_of(node_index).unwrap();

		// ? If the node to remove has direct children, move them next to the node in the layout
		self.move_direct_children_next_to(node_index, node_index, &RelativePosition::After);

		// ? Remove node
		self.remove_node_from_list(node_index);

		// ? Detach from parent and remove parent if less than 2 children
		let mut should_remove_parent = false;
		if let Some(Some(parent_node)) = self.nodes.get_mut(parent_node_index) {
			if let LayoutNodeType::Container(ref mut parent_container_data) = parent_node.node_type {

				// ? Remove the child from the parent
				parent_container_data.remove(node_index);

				// ? Mark parent as to be removed if < 2 children and is not the root
				if parent_container_data.len() < 2 && parent_node_index != 0 {
					should_remove_parent = true;
				} else {
					index_of_container_to_rebalance = Some(parent_node_index);
				}
			}
		}

		// ? Remove parent if marked as so
		if should_remove_parent {
			self.remove_node(parent_node_index);
		}

		// ? Return the index of the top node to rebalance
		index_of_container_to_rebalance
	}

	/// Removes the subtree from the layout
	pub fn remove_subtree(&mut self, subtree_root_index: NodeIndex) -> Option<NodeIndex> {
		// ? Can't remove root node
		if subtree_root_index == 0 {
			error!("Tried to remove root index of layout!");
			return None;
		}

		// ? Validate the provided node index
		if !(0 < subtree_root_index && subtree_root_index < self.nodes.len()) || self.nodes[subtree_root_index].is_none() {
			error!("Tried to remove invalid node index '{}' from layout!", subtree_root_index);
			return None;
		}

		let mut index_of_container_to_rebalance = None;

		// ? Remove all childrens if any
		self.remove_all_children_of(subtree_root_index);

		// ? Remove node (keep hook on parent index for further detach)
		let parent_node_index = self.get_parent_node_index_of(subtree_root_index).unwrap();
		self.remove_node_from_list(subtree_root_index);

		// ? Detach from parent and remove parent if less than 2 children
		let mut should_remove_parent = false;
		if let Some(Some(parent_node)) = self.nodes.get_mut(parent_node_index) {
			if let LayoutNodeType::Container(ref mut container_node_data) = parent_node.node_type {

				// ? Remove the child from the parent
				container_node_data.remove(subtree_root_index);

				// ? Mark parent as to be removed if < 2 children and is not the root
				if container_node_data.len() < 2 && parent_node_index != 0 {
					should_remove_parent = true;
				} else {
					index_of_container_to_rebalance = Some(parent_node_index);
				}
			}
		}

		// ? Remove parent if marked as so
		if should_remove_parent {
			self.remove_node(parent_node_index);
		}

		// ? Return the index of the top node to rebalance
		index_of_container_to_rebalance
	}

	/// Returns true if the layout contains the window index.
	pub fn intersects_with_window_area(&self, area: &Area) -> bool {
		if let Some(Some(root_node)) = self.nodes.get(0) {
			root_node.area.intersection(*area) != IntersectionResult::NoIntersection
		} else {
			false
		}
	}

	/// Returns true if the provided node index points to a container node.
	fn is_container_node(&self, node_index: NodeIndex) -> bool {
		if let Some(Some(node)) = self.nodes.get(node_index) {
			match node.node_type {
				LayoutNodeType::Window(_) => return false,
				LayoutNodeType::Container(_) => return true,
			}
		}

		false
	}

	/// Recursive method to print the subtree to the console, takes the prefix to add to the line to print (level of the tree).
	fn print_subtree_to_console_recur(&self, subtree_root_index: NodeIndex, prefix: String) {
		let children_indices = self.get_direct_children_indices_of(subtree_root_index);
		for (i, &child_index) in children_indices.iter().enumerate() {
			let is_last_child = i == children_indices.len() - 1;
			let is_container_node = self.is_container_node(child_index);
			match (is_last_child, is_container_node) {
				(false, false) => println!("{}├ W-{}", prefix, child_index),
				(false, true) => {
					let container_axis = self.get_axis_of(child_index).unwrap();
					let arrow_character = match container_axis {
						LayoutAxis::Vertical => "▼",
						LayoutAxis::Horizontal => "►",
					};
					println!("{}├ C-{} {}", prefix, child_index, arrow_character);
					self.print_subtree_to_console_recur(child_index, format!("{}│", prefix))
				},
				(true, false) => println!("{}└ W-{}", prefix, child_index),
				(true, true) => {
					let container_axis = self.get_axis_of(child_index).unwrap();
					let arrow_character = match container_axis {
						LayoutAxis::Vertical => "▼",
						LayoutAxis::Horizontal => "►",
					};
					println!("{}└ C-{} {}", prefix, child_index, arrow_character);
					self.print_subtree_to_console_recur(child_index, format!("{} ", prefix))
				},
			}
		}
	}

	/// Prints the subtree from the provided root to the console.
	pub fn print_subtree_to_console(&self, subtree_root_index: NodeIndex) {
		let container_axis = self.get_axis_of(subtree_root_index).unwrap();
		let arrow_character = match container_axis {
			LayoutAxis::Vertical => "▼",
			LayoutAxis::Horizontal => "►",
		};
		println!("C-{} {}", subtree_root_index, arrow_character);
		self.print_subtree_to_console_recur(subtree_root_index, String::from(""));
	}
}
