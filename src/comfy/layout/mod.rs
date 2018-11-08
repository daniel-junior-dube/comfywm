use std::collections::HashMap;

use wlroots::{Area, IntersectionResult, Origin, Size, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle};

use compositor::window::Window;
use utils::handle_helper::shell_handle_helper;

/*
..####....####...##..##...####...######...####...##..##..######...####..
.##..##..##..##..###.##..##........##....##..##..###.##....##....##.....
.##......##..##..##.###...####.....##....######..##.###....##.....####..
.##..##..##..##..##..##......##....##....##..##..##..##....##........##.
..####....####...##..##...####.....##....##..##..##..##....##.....####..
........................................................................
*/

/// Alias for the type of a node index
type NodeIndex = usize;

/// The root of the layout is always the first node in the list.
const INDEX_OF_ROOT: NodeIndex = 0;

/// Axis on which a layout node operates.
#[derive(Clone, Eq, PartialEq)]
pub enum LayoutAxis {
	Vertical,
	Horizontal,
}
impl LayoutAxis {
	/// Returns the characters which represents the axis.
	pub fn get_direction_char(&self) -> char {
		match self {
			LayoutAxis::Vertical => '▼',
			LayoutAxis::Horizontal => '►',
		}
	}
}

/// Direction used when interacting on the layout. (Example: Moving or adding in a relative direction from a node)
#[derive(Clone, Eq, PartialEq)]
pub enum LayoutDirection {
	Up,
	Down,
	Left,
	Right,
}
impl LayoutDirection {
	/// Returns the axis associated with the instance of the layout direction.
	pub fn get_axis(&self) -> LayoutAxis {
		match self {
			LayoutDirection::Up | LayoutDirection::Down => LayoutAxis::Vertical,
			LayoutDirection::Left | LayoutDirection::Right => LayoutAxis::Horizontal,
		}
	}

	pub fn get_relative_position(&self) -> RelativePosition {
		match self {
			LayoutDirection::Up | LayoutDirection::Left => RelativePosition::Before,
			LayoutDirection::Down | LayoutDirection::Right => RelativePosition::After,
		}
	}
}

/// Relative linear position used when obtaining a node that is before or after another.
pub enum RelativePosition {
	After,
	Before,
}

/*
.##.......####...##..##...####...##..##..######..##..##...####...#####...######.
.##......##..##...####...##..##..##..##....##....###.##..##..##..##..##..##.....
.##......######....##....##..##..##..##....##....##.###..##..##..##..##..####...
.##......##..##....##....##..##..##..##....##....##..##..##..##..##..##..##.....
.######..##..##....##.....####....####.....##....##..##...####...#####...######.
................................................................................
.#####...##..##..######..##......#####...######..#####..
.##..##..##..##....##....##......##..##..##......##..##.
.#####...##..##....##....##......##..##..####....#####..
.##..##..##..##....##....##......##..##..##......##..##.
.#####....####...######..######..#####...######..##..##.
........................................................
*/

/// Helper structure to create an instance of a LayoutNode.
struct LayoutNodeBuilder {
	parent_node_index: NodeIndex,
	children_indices: Vec<NodeIndex>,
	axis: LayoutAxis,
	area: Area,
	weight: f32,
	need_rebalancing: bool,
}
impl LayoutNodeBuilder {
	/// Create a new node builder.
	pub fn new() -> Self {
		LayoutNodeBuilder {
			parent_node_index: 0,
			children_indices: vec![],
			axis: LayoutAxis::Horizontal,
			area: Area::new(Origin::new(0, 0), Size::new(0, 0)),
			weight: 1.0,
			need_rebalancing: false,
		}
	}

	/// Sets the parent index.
	pub fn parent_index(mut self, parent_index: NodeIndex) -> Self {
		self.parent_node_index = parent_index;
		self
	}

	/// Set the axis.
	pub fn axis(mut self, axis: LayoutAxis) -> Self {
		self.axis = axis;
		self
	}

	/// Set the area.
	pub fn area(mut self, area: Area) -> Self {
		self.area = area;
		self
	}

	/// Builds an instance of LayoutNode using the attributes set on the builder.
	pub fn build(&self) -> LayoutNode {
		LayoutNode::new(
			self.parent_node_index,
			self.children_indices.clone(),
			self.axis.clone(),
			self.area,
			self.weight,
			self.need_rebalancing,
		)
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

/// A layout node represents a node in the layout tree.
#[derive(Clone)]
pub struct LayoutNode {
	parent_node_index: NodeIndex,
	children_indices: Vec<NodeIndex>,
	axis: LayoutAxis,
	area: Area,
	weight: f32,
	need_rebalancing: bool,
}
impl LayoutNode {
	/// Create a new node with the itself as it's parent.
	pub fn new_root_node(area: Area) -> Self {
		LayoutNodeBuilder::new().area(area).build()
	}

	/// Create a new node with the provided node type and binds it to the provided parent node index.
	pub fn new(
		parent_node_index: NodeIndex,
		children_indices: Vec<NodeIndex>,
		axis: LayoutAxis,
		area: Area,
		weight: f32,
		need_rebalancing: bool,
	) -> Self {
		LayoutNode {
			parent_node_index,
			children_indices,
			axis,
			area,
			weight,
			need_rebalancing,
		}
	}

	/// Returns true if the node has no children.
	pub fn is_leaf(&self) -> bool {
		self.children_indices.is_empty()
	}

	/// Returns true if the node intersects the given area.
	pub fn intersects(&self, area: &Area) -> bool {
		self.area.intersection(*area) != IntersectionResult::NoIntersection
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

	/// Adds the provided index as a child of the container right after .
	fn add_child_index_next_to(
		&mut self,
		index_to_add: NodeIndex,
		value_of_target: NodeIndex,
		position: &RelativePosition,
	) {
		if let Some(index_of_target) = self.index_of(value_of_target) {
			match position {
				RelativePosition::After => self.children_indices.insert(index_of_target + 1, index_to_add),
				RelativePosition::Before => self.children_indices.insert(index_of_target, index_to_add),
			}
		}
	}

	/// If `child_value` exists as a child of the container, returns it's index in the container.
	fn index_of(&self, child_value: NodeIndex) -> Option<NodeIndex> {
		self.children_indices.iter().position(|element| *element == child_value)
	}

	/// If `child_value` exists as a child of the node, removes it.
	fn remove(&mut self, child_value: NodeIndex) {
		if let Some(index_of_child) = self.index_of(child_value) {
			self.children_indices.remove(index_of_child);
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
	/// Key value storage which pairs the index of the node in the tree and the window.
	leaf_index_to_windows_map: HashMap<NodeIndex, Window>,
	/// Tree data structure that builds the areas for the windows and container
	layout_tree: RegionBasedKAryLayoutTree,
}

impl Layout {
	/// Create a new layout that occupates the space of the provided area
	pub fn new(output_area: Area) -> Self {
		Layout {
			leaf_index_to_windows_map: HashMap::new(),
			layout_tree: RegionBasedKAryLayoutTree::new(output_area),
		}
	}

	/// Returns a vector containing the windows indices of a windows contained in this layout.
	/// TODO: Could put the windows in cache and update the cache only after a modification
	pub fn get_windows(&self) -> Vec<Window> {
		self
			.leaf_index_to_windows_map
			.iter()
			.map(|(_node_index, window)| window.clone())
			.collect()
	}

	/// Updates the render area of the layout.
	/// Does not rebalances the tree.
	pub fn update_area(&mut self, area: Area) {
		self.layout_tree.update_area(area)
	}

	pub fn rebalance_tree(&mut self) {
		self.layout_tree.rebalance();
	}

	/// Updates the area of the layout then rebalances the tree from the root.
	pub fn update_area_and_rebalance(&mut self, area: Area) {
		self.update_area(area);
		self.rebalance_tree();
	}

	/// Returns the render area of the layout.
	pub fn area(&self) -> Option<Area> {
		self.layout_tree.area()
	}

	/// Returns the shell handle of the active node if any.
	pub fn get_active_shell_handle(&self) -> Option<WLRXdgV6ShellSurfaceHandle> {
		if let Some(Window { shell_handle, .. }) = self.leaf_index_to_windows_map.get(&self.layout_tree.active_node_index) {
			Some(shell_handle.clone())
		} else {
			None
		}
	}

	/// Returns the index of the node containing the provided xdg shell surface handle.
	pub fn index_of_node_containing_shell_handle(&self, shell_handle: &WLRXdgV6ShellSurfaceHandle) -> Option<NodeIndex> {
		let shell_handle_area = shell_handle_helper::get_shell_area(shell_handle);
		let intersecting_node_indices = self.layout_tree.indices_of_intersecting_leaves(&shell_handle_area);
		intersecting_node_indices.iter().cloned().find(|node_index| {
			if let Some(window) = self.leaf_index_to_windows_map.get(&node_index) {
				window.shell_handle == *shell_handle
			} else {
				false
			}
		})
	}

	fn rebalance(&mut self) {
		let indices_of_resized_nodes = self.layout_tree.rebalance();
		for index_of_resized_node in indices_of_resized_nodes.iter() {
			if let Some(window) = self.leaf_index_to_windows_map.get_mut(index_of_resized_node) {
				let node_area = self.layout_tree.get_node_area(*index_of_resized_node).unwrap();
				window.resize(&node_area);
			}
		}
	}

	/// Adds a window in the layout given it's associated xdg shell surface handle.
	/// The containing node will be a neighbor of the currently activated node if any.
	/// Otherwise, it will be added as a child of the root.
	pub fn add_window(
		&mut self,
		window: Window,
		direction: &LayoutDirection,
		set_as_last_activated: bool,
		rebalance_after_insertion: bool,
	) -> Result<(), String> {
		let index_of_new_node = self
			.layout_tree
			.add_new_empty_node(LayoutAxis::Horizontal, INDEX_OF_ROOT);
		let index_of_parent_option = if self.layout_tree.active_node_index == INDEX_OF_ROOT {
			self.layout_tree.move_index_under_root(index_of_new_node)
		} else {
			self
				.layout_tree
				.move_index_relative_to_active_node(index_of_new_node, direction)
		};

		// ? If a parent was return, we set the window node as activated, else we undo the insertion.
		if index_of_parent_option.is_some() {
			if set_as_last_activated {
				self.layout_tree.set_as_last_activated(index_of_new_node);
			}
			self.leaf_index_to_windows_map.insert(index_of_new_node, window);
			if rebalance_after_insertion {
				self.rebalance();
			}
			self.layout_tree.print_to_console();
		} else {
			self.layout_tree.remove_node(index_of_new_node)?;
		}

		Ok(())
	}

	/// If the layout contains a window associated with the provided xdg shell surface handle, we remove it from the layout.
	/// Returns the node index of the root of the subtree that needs to be rebalanced.
	pub fn remove_window_from_shell_handle(
		&mut self,
		shell_handle: &WLRXdgV6ShellSurfaceHandle,
		rebalance_after_removal: bool,
	) -> Result<(), String> {
		if let Some(index_of_node_containing_shell) = self.index_of_node_containing_shell_handle(shell_handle) {
			let removed_leaves = self.layout_tree.remove_node(index_of_node_containing_shell)?;

			// ? If no leaf was removed, there must be a mistake
			if removed_leaves.is_empty() {
				return Err("No removed leaves was returned from `layout_tree.remove_node`".to_string());
			}

			// ? Removes the window associated with the node index if any.
			for index_of_removed_leaf in removed_leaves.iter() {
				self.leaf_index_to_windows_map.remove(&index_of_removed_leaf);
			}

			// ? Rebalance if desired
			if rebalance_after_removal {
				self.rebalance();
			}
			self.layout_tree.print_to_console();
		} else {
			return Err("Tried to remove a window which is not contained in the layout.".to_string());
		}
		Ok(())
	}

	/// Returns true if the layout contains the window index.
	pub fn intersects_with(&self, area: &Area) -> bool {
		self.layout_tree.intersects_with(area)
	}
}

/*
.#####...######...####...######...####...##..##..#####....####....####...######..#####..
.##..##..##......##........##....##..##..###.##..##..##..##..##..##......##......##..##.
.#####...####....##.###....##....##..##..##.###..#####...######...####...####....##..##.
.##..##..##......##..##....##....##..##..##..##..##..##..##..##......##..##......##..##.
.##..##..######...####...######...####...##..##..#####...##..##...####...######..#####..
........................................................................................
.##..##...####...#####...##..##..##.......####...##..##...####...##..##..######..######..#####...######..######.
.##.##...##..##..##..##...####...##......##..##...####...##..##..##..##....##......##....##..##..##......##.....
.####....######..#####.....##....##......######....##....##..##..##..##....##......##....#####...####....####...
.##.##...##..##..##..##....##....##......##..##....##....##..##..##..##....##......##....##..##..##......##.....
.##..##..##..##..##..##....##....######..##..##....##.....####....####.....##......##....##..##..######..######.
................................................................................................................
*/

struct RegionBasedKAryLayoutTree {
	/// Array of availables empty spots in the `nodes`.
	available_places: Vec<NodeIndex>,
	/// Array of layout node.
	nodes: Vec<Option<LayoutNode>>,
	/// Index of the active node in the layout.
	active_node_index: NodeIndex,
}

impl RegionBasedKAryLayoutTree {
	/// Create a new layout that occupates the space of the provided area
	pub fn new(output_area: Area) -> Self {
		let root_node = LayoutNode::new_root_node(output_area);
		RegionBasedKAryLayoutTree {
			available_places: vec![],
			nodes: vec![Some(root_node)],
			active_node_index: INDEX_OF_ROOT,
		}
	}

	/// Updates the render area of the layout.
	/// Does not rebalances the tree.
	pub fn update_area(&mut self, area: Area) {
		if let Some(ref mut root_node) = self.nodes[INDEX_OF_ROOT] {
			root_node.area = area;
		}
	}

	/// Returns the render area of the layout.
	pub fn area(&self) -> Option<Area> {
		if let Some(ref root_node) = self.nodes[INDEX_OF_ROOT] {
			Some(root_node.area)
		} else {
			None
		}
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
			Some(node.area)
		} else {
			None
		}
	}

	/// Returns the weight sum of all the direct children of the provided node.
	fn get_direct_children_weight_sum(&self, node_index: NodeIndex) -> Option<f32> {
		if let Some(Some(node)) = self.nodes.get(node_index).clone() {
			return Some(
				node
					.children_indices
					.iter()
					.map(|index| match self.nodes[*index] {
						Some(ref layout_node) => layout_node.weight,
						None => 0.0,
					}).sum(),
			);
		}
		None
	}

	/// Returns a clone of a specific node associated with the provided node index if any.
	fn get_node_clone(&mut self, node_index: NodeIndex) -> Option<LayoutNode> {
		if let Some(Some(node)) = self.nodes.get(node_index) {
			Some(node.clone())
		} else {
			None
		}
	}

	/// Recalculates the position and dimensions of each node from the root.
	pub fn rebalance(&mut self) -> Vec<NodeIndex> {
		self.rebalance_subtree(INDEX_OF_ROOT)
	}

	/// Recalculates the position and dimensions of each node from the provided node index.
	/// Returns a vector containing all the indices of the leaves that changed.
	fn rebalance_subtree(&mut self, subtree_root_node: NodeIndex) -> Vec<NodeIndex> {
		let mut dirty_leaves = Vec::new();
		let mut indices_of_nodes_to_rebalance = vec![subtree_root_node];
		while let Some(index_of_node_to_rebalance) = indices_of_nodes_to_rebalance.pop() {
			if let Some(clone_of_node_to_rebalance) = self.get_node_clone(index_of_node_to_rebalance) {
				let children_weight_sum: f32 = self.get_direct_children_weight_sum(index_of_node_to_rebalance).unwrap();
				let mut origin_offset: i32 = 0;
				for child_index in clone_of_node_to_rebalance.children_indices {
					if let Some(Some(child_node)) = self.nodes.get_mut(child_index) {
						child_node.rebalance_area(
							&clone_of_node_to_rebalance.area,
							&clone_of_node_to_rebalance.axis,
							children_weight_sum,
							origin_offset,
						);

						match clone_of_node_to_rebalance.axis {
							LayoutAxis::Horizontal => {
								origin_offset += child_node.area.size.width;
							}
							LayoutAxis::Vertical => {
								origin_offset += child_node.area.size.height;
							}
						}

						if child_node.is_leaf() {
							dirty_leaves.push(child_index);
						} else {
							indices_of_nodes_to_rebalance.push(child_index)
						}
					}
				}
			}
		}
		dirty_leaves
	}

	/// Move the node associated to the provided node index as the last child of a provided target.
	fn move_index_under(&mut self, node_index: NodeIndex, index_of_target: NodeIndex) -> Option<NodeIndex> {
		// ? Remove child from current parent
		if let Some(parent_node_index) = self.get_parent_node_index_of(node_index) {
			if let Some(Some(parent_node)) = self.nodes.get_mut(parent_node_index) {
				parent_node.remove(node_index);
			}
		}

		// ? Set parent to child
		if let Some(Some(node)) = self.nodes.get_mut(node_index) {
			node.parent_node_index = index_of_target;
		}

		// ? Set child to parent
		if let Some(Some(parent_node)) = self.nodes.get_mut(index_of_target) {
			parent_node.add_child_index(node_index);
			return Some(index_of_target);
		}
		None
	}

	/// Moves the nodes associated with the provided node indices next to the target.
	fn move_indices_next_to(
		&mut self,
		indices_to_move: Vec<NodeIndex>,
		index_of_target: NodeIndex,
		relative_position: &RelativePosition,
	) {
		for index_to_move in indices_to_move.iter().rev() {
			self.move_index_next_to(*index_to_move, index_of_target, relative_position);
		}
	}

	/// Moves the nodes associated with the direct children of the provided `parent_node_index` next to the target.
	fn move_direct_children_next_to(
		&mut self,
		parent_node_index: NodeIndex,
		index_of_target: NodeIndex,
		relative_position: &RelativePosition,
	) {
		let direct_children_indices = self.get_direct_children_indices_of(parent_node_index);
		self.move_indices_next_to(direct_children_indices, index_of_target, relative_position);
	}

	/// Moves a node in the layout next to another.
	fn move_index_next_to(
		&mut self,
		index_of_node_to_move: NodeIndex,
		index_of_target: NodeIndex,
		relative_position: &RelativePosition,
	) -> Option<NodeIndex> {
		// ? Get parent node index, otherwise print an error (parent of target has to exist or target isn't a valid node)
		if let Some(index_of_parent_of_target) = self.get_parent_node_index_of(index_of_target) {
			// ? Remove node to add from it's parent
			if let Some(parent_index_of_node_to_add) = self.get_parent_node_index_of(index_of_node_to_move) {
				if let Some(Some(parent_of_node_to_add)) = self.nodes.get_mut(parent_index_of_node_to_add) {
					parent_of_node_to_add.remove(index_of_node_to_move);
				}
			}

			// ? Set parent to child
			if let Some(Some(node)) = self.nodes.get_mut(index_of_node_to_move) {
				node.parent_node_index = index_of_parent_of_target;
			}

			// ? Set child to parent
			if let Some(Some(parent_node)) = self.nodes.get_mut(index_of_parent_of_target) {
				parent_node.add_child_index_next_to(index_of_node_to_move, index_of_target, relative_position);
				return Some(index_of_parent_of_target);
			}
		} else {
			error!(
				"INVALID TARGET NODE: Tried to add index '{}' next to '{}', but target has no parent.",
				index_of_node_to_move, index_of_target
			);
		}
		None
	}

	fn get_axis_of(&self, node_index: NodeIndex) -> Option<LayoutAxis> {
		if let Some(Some(node)) = self.nodes.get(node_index) {
			Some(node.axis.clone())
		} else {
			None
		}
	}

	/// Adds a new empty node to the layout, as the last child of the provided parent_node_index.
	fn add_new_empty_node(&mut self, axis: LayoutAxis, parent_node_index: NodeIndex) -> NodeIndex {
		let layout_node_builder = LayoutNodeBuilder::new().parent_index(parent_node_index).axis(axis);

		self.add_node_to_list(layout_node_builder.build())
	}

	/// Binds the provided index as a neighbor of the active node.
	/// Returns the index of the parent node if if was successfully added.
	fn move_index_relative_to_active_node(
		&mut self,
		index_of_node_to_add: NodeIndex,
		direction: &LayoutDirection,
	) -> Option<NodeIndex> {
		let active_node_is_root = self.active_node_index == INDEX_OF_ROOT;
		let parent_node_index = if active_node_is_root {
			0
		} else {
			self
				.get_parent_node_index_of_active_node()
				.unwrap_or_else(|| INDEX_OF_ROOT)
		};

		let axis_of_parent = self.get_axis_of(parent_node_index).unwrap();
		let nb_children_of_parent = self.get_direct_children_indices_of(parent_node_index).len();
		match (nb_children_of_parent, direction.get_axis() == axis_of_parent) {
			// ? Parent has only 0 or 1 children, we don't care about the axis since where going to change it
			(0...1, _) => {
				if active_node_is_root {
					// ? Active node is root, just add new index under it
					self.move_index_under_root(index_of_node_to_add);
				} else {
					// ? Move the new node before or after the active node (depending on the given direction)
					let active_node_index = self.active_node_index;
					match direction {
						LayoutDirection::Left | LayoutDirection::Up => {
							self.move_index_next_to(index_of_node_to_add, active_node_index, &RelativePosition::Before)
						}
						LayoutDirection::Right | LayoutDirection::Down => {
							self.move_index_next_to(index_of_node_to_add, active_node_index, &RelativePosition::After)
						}
					};
				}

				// ? Change the axis of the parent node
				if let Some(Some(parent_node)) = self.nodes.get_mut(parent_node_index) {
					parent_node.set_axis(direction.get_axis());
				}

				return Some(parent_node_index);
			}

			// ? Parent has more than 1 child, but is on the same axis
			(_, true) => {
				let active_node_index = self.active_node_index;
				match direction {
					LayoutDirection::Left | LayoutDirection::Up => {
						self.move_index_next_to(index_of_node_to_add, active_node_index, &RelativePosition::Before)
					}
					LayoutDirection::Right | LayoutDirection::Down => {
						self.move_index_next_to(index_of_node_to_add, active_node_index, &RelativePosition::After)
					}
				};
				return Some(parent_node_index);
			}

			// ? Parent has more than 1 child, but is on a different axis
			(_, false) => {
				let active_node_index = self.active_node_index;

				// ? Creates new container node to which we will add the active node and the new node
				let new_container_index = self.add_new_empty_node(direction.get_axis(), parent_node_index);

				// ? Move the container next to the active node
				self.move_index_next_to(new_container_index, active_node_index, &RelativePosition::After);

				// ? Move the active node under the new container
				self.move_index_under(active_node_index, new_container_index);

				// ? Move the new node before or after the active node index (depending on the given direction)
				let relative_position_where_to_move = direction.get_relative_position();
				self.move_index_next_to(
					index_of_node_to_add,
					active_node_index,
					&relative_position_where_to_move,
				);
				return Some(parent_node_index);
			}
			_ => {
				// TODO: ERROR
			}
		}

		None
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

	/// Return the index of the node that would be the active one in the case of the active node being deleted.
	pub fn find_fallback_node_index(&self, node_index: NodeIndex) -> NodeIndex {
		if let Some(parent_node_index) = self.get_parent_node_index_of(node_index) {
			// ? Use left sibling or right sibling as fallback
			if let Some(Some(ref parent_node)) = self.nodes.get(parent_node_index) {
				let index_of_child = parent_node.index_of(node_index).unwrap();

				// ? Left sibling
				if index_of_child > 0 {
					if let Some(previous_sibling_index) = parent_node.children_indices.get(index_of_child - 1) {
						return *previous_sibling_index;
					}
				}

				// ? Right sibling
				if index_of_child < parent_node.children_indices.len() - 1 {
					if let Some(next_sibling_index) = parent_node.children_indices.get(index_of_child + 1) {
						return *next_sibling_index;
					}
				}
			}

			// ? If parent has no other child and parent is root_node
			if parent_node_index == 0 {
				return 0;
			}

			// ? Check for a fallback for the parent
			self.find_fallback_node_index(parent_node_index)
		} else {
			0
		}
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

	/// Returns true if the provided node index is associates with an existing node in the node list.
	/// In other words, if there is an instance of a node at the index.
	fn node_exists(&self, node_index: NodeIndex) -> bool {
		if let Some(Some(_node)) = self.nodes.get(node_index) {
			true
		} else {
			false
		}
	}

	/// Removes the provided node index from the list of nodes.
	/// If the node is the currently active one, assign the active to the fallback node.
	/// Also removes trailing holes dynamically if the removed node is the last one in the list.
	fn remove_node_from_list(&mut self, node_index: NodeIndex) -> Result<(), String> {
		// ? Check that the node is valid
		if !self.node_exists(node_index) {
			return Err("Tried to remove unexistant node index from the list".to_string());
		}

		// ? If the node is the active one, set active to fallback.
		if node_index == self.active_node_index {
			self.active_node_index = self.find_fallback_node_index(node_index);
		}

		// ? If the node index to remove is the last of the list, clear trailing holes.
		self.nodes[node_index] = None;

		// ? If the removed node is the last of the list, clear trailing holes. Otherwise create one.
		if node_index == self.nodes.len() - 1 {
			self.clear_node_list_trailing_holes();
		} else {
			self.available_places.push(node_index);
		}

		Ok(())
	}

	/// Returns a vector containing all the indices of the direct children of a node.
	fn get_direct_children_indices_of(&self, node_index: NodeIndex) -> Vec<NodeIndex> {
		let mut direct_children_of = Vec::new();
		if let Some(Some(node)) = self.nodes.get(node_index) {
			for child_value in node.children_indices.iter() {
				direct_children_of.push(*child_value)
			}
		}
		direct_children_of
	}

	/// Removes the node associated with the provided node index from the layout.
	/// Will remove all it's children and it's parent if it has less than 2 children after removal of the child.
	/// Returns the index of the top node which needs to be rebalanced.
	pub fn remove_node(&mut self, node_index: NodeIndex) -> Result<Vec<NodeIndex>, String> {
		let mut removed_leaves = Vec::new();

		// ? Can't remove root node
		if node_index == 0 {
			return Err("Tried to remove root index of layout!".to_string());
		}

		// ? Validate the provided node index (can't remove invalid node)
		if !(0 < node_index && node_index < self.nodes.len()) || self.nodes[node_index].is_none() {
			return Err(format!("Tried to remove node index '{}' from layout!", node_index));
		}

		// ? keep hook on parent index for further detach
		let parent_node_index = self.get_parent_node_index_of(node_index).unwrap();

		// ? If the node to remove has direct children, move them next to the node in the layout
		self.move_direct_children_next_to(node_index, node_index, &RelativePosition::After);

		// ? Remove node
		self.remove_node_from_list(node_index)?;
		removed_leaves.push(node_index);

		// ? Detach from parent and remove parent if less than 2 children
		let mut should_remove_parent = false;
		if let Some(Some(parent_node)) = self.nodes.get_mut(parent_node_index) {
			// ? Remove the child from the parent
			parent_node.remove(node_index);

			// ? Mark parent as to be removed if < 2 children and is not the root
			if parent_node.len() < 2 && parent_node_index != 0 {
				should_remove_parent = true;
			}
		}

		// ? Remove parent if marked as so
		if should_remove_parent {
			self.remove_node(parent_node_index)?;
		} else {
			self.toggle_rebalance_flag(parent_node_index, true);
		}

		// ? Return the index of the top node to rebalance
		Ok(removed_leaves)
	}

	/// Sets the value of the `need_rebalance` flag of a node
	pub fn toggle_rebalance_flag(&mut self, node_index: NodeIndex, flag_value: bool) {
		if let Some(Some(node)) = self.nodes.get_mut(node_index) {
			node.need_rebalancing = flag_value;
		}
	}

	/// Returns true if the layout contains the window index.
	pub fn intersects_with(&self, area: &Area) -> bool {
		if let Some(Some(root_node)) = self.nodes.get(0) {
			root_node.area.intersection(*area) != IntersectionResult::NoIntersection
		} else {
			false
		}
	}

	/// Returns a vector containing the indices of all the leaves that intersects with the provided area.
	pub fn indices_of_intersecting_leaves(&self, area: &Area) -> Vec<NodeIndex> {
		let mut intersecting_leaves = Vec::new();
		let mut indices_of_nodes_to_check = vec![INDEX_OF_ROOT];
		while let Some(index_of_node_to_check) = indices_of_nodes_to_check.pop() {
			if let Some(Some(node_to_check)) = self.nodes.get(index_of_node_to_check) {
				if node_to_check.is_leaf() && node_to_check.intersects(area) {
					intersecting_leaves.push(index_of_node_to_check);
				} else {
					node_to_check
						.children_indices
						.iter()
						.for_each(|&child_index| indices_of_nodes_to_check.push(child_index))
				}
			}
		}
		intersecting_leaves
	}

	/// Returns true if the provided node index points to a container node.
	fn is_leaf_node(&self, node_index: NodeIndex) -> bool {
		if let Some(Some(node)) = self.nodes.get(node_index) {
			node.is_leaf()
		} else {
			false
		}
	}

	/// Recursive method to print the subtree to the console, takes the prefix to add to the line to print (level of the tree).
	fn print_subtree_to_console_recur(&self, subtree_root_index: NodeIndex, prefix: &str) {
		let children_indices = self.get_direct_children_indices_of(subtree_root_index);
		for (i, &child_index) in children_indices.iter().enumerate() {
			let is_last_child = i == children_indices.len() - 1;
			let is_leaf_node = self.is_leaf_node(child_index);
			match (is_last_child, is_leaf_node) {
				(false, true) => println!("{}├ W-{}", prefix, child_index),
				(false, false) => {
					let container_axis = self.get_axis_of(child_index).unwrap();
					let direction_character = container_axis.get_direction_char();
					println!("{}├ C-{} {}", prefix, child_index, direction_character);
					self.print_subtree_to_console_recur(child_index, &format!("{}│", prefix))
				}
				(true, true) => println!("{}└ W-{}", prefix, child_index),
				(true, false) => {
					let container_axis = self.get_axis_of(child_index).unwrap();
					let direction_character = container_axis.get_direction_char();
					println!("{}└ C-{} {}", prefix, child_index, direction_character);
					self.print_subtree_to_console_recur(child_index, &format!("{} ", prefix))
				}
			}
		}
	}

	/// Prints the subtree from the provided root to the console.
	pub fn print_subtree_to_console(&self, subtree_root_index: NodeIndex) {
		let container_axis = self.get_axis_of(subtree_root_index).unwrap();
		let direction_character = container_axis.get_direction_char();
		println!("C-{} {}", subtree_root_index, direction_character);
		self.print_subtree_to_console_recur(subtree_root_index, "");
	}

	// TODO: Implement the `debug` trait to print the layout tree from println! directly
	/// Prints the tree to the console.
	pub fn print_to_console(&self) {
		self.print_subtree_to_console(INDEX_OF_ROOT);
	}

	/*
	.##...##..######..#####..
	.##...##....##....##..##.
	.##.#.##....##....#####..
	.#######....##....##.....
	..##.##...######..##.....
	.........................
	*/

	/// Removes the subtree from the layout
	pub fn _remove_subtree(&mut self, subtree_root_index: NodeIndex) -> Result<Vec<NodeIndex>, String> {
		let mut removed_leaves = Vec::new();

		// ? Can't remove root node
		if subtree_root_index == 0 {
			return Err("Tried to remove root index of layout!".to_string());
		}

		// ? Validate the provided node index
		if !(0 < subtree_root_index && subtree_root_index < self.nodes.len()) || self.nodes[subtree_root_index].is_none() {
			return Err(format!(
				"Tried to remove subtree root index '{}' from layout!",
				subtree_root_index
			));
		}

		// ? Remove all childrens if any
		let mut temp = self._remove_all_children_of(subtree_root_index);
		removed_leaves.append(&mut temp);

		// ? Remove node (keep hook on parent index for further detach)
		let parent_node_index = self.get_parent_node_index_of(subtree_root_index).unwrap();
		self.remove_node_from_list(subtree_root_index)?;

		// ? Detach from parent and remove parent if less than 2 children
		let mut should_remove_parent = false;
		if let Some(Some(parent_node)) = self.nodes.get_mut(parent_node_index) {
			// ? Remove the child from the parent
			parent_node.remove(subtree_root_index);

			// ? Mark parent as to be removed if < 2 children and is not the root
			if parent_node.len() < 2 && parent_node_index != 0 {
				should_remove_parent = true;
			}
		}

		// ? Remove parent if marked as so
		if should_remove_parent {
			self.remove_node(parent_node_index)?;
		} else {
			self.toggle_rebalance_flag(parent_node_index, true);
		}

		Ok(removed_leaves)
	}

	/// Removes all the children of a subtree from the layout.
	/// Since we want to remove all children, it doesn't unassign subchildren from parents except for the subtree root.
	pub fn _remove_all_children_of(&mut self, subtree_root_index: NodeIndex) -> Vec<NodeIndex> {
		let mut indices_of_removed_leaves = Vec::new();
		let indices_of_nodes_to_remove = self._get_indices_of_subtree(subtree_root_index, false, true);

		// ? Remove all children from the subtree iteratively
		for index_of_node_to_remove in indices_of_nodes_to_remove.iter() {
			if self.is_leaf_node(*index_of_node_to_remove) {
				indices_of_removed_leaves.push(*index_of_node_to_remove);
			}
			self.remove_node_from_list(*index_of_node_to_remove);
		}

		// ? If the subtree root is a container, clear the children
		if let Some(Some(node)) = self.nodes.get_mut(subtree_root_index) {
			node.children_indices.clear();
		}
		indices_of_removed_leaves
	}

	/// Returns all the node indices of the subtree from the provided root.
	/// If `only_leafs` is true, skips all container nodes.
	pub fn _get_indices_of_subtree(
		&self,
		subtree_root_index: NodeIndex,
		only_leafs: bool,
		exclude_root: bool,
	) -> Vec<NodeIndex> {
		let mut indices_of_subtree = vec![];

		// ? Iteratively add all node indices to the indices_of_subtree
		let mut indices_to_check = vec![subtree_root_index];
		while let Some(node_index) = indices_to_check.pop() {
			if let Some(ref node) = self.nodes[node_index] {
				if node.is_leaf() {
					indices_of_subtree.push(node_index);
				} else {
					if !only_leafs {
						indices_of_subtree.push(node_index);
					}

					// ? Add all child indices to the indices to check
					for i in node.children_indices.iter() {
						indices_to_check.push(*i);
					}
				}
			}
		}
		if exclude_root {
			indices_of_subtree.remove(0);
		}
		indices_of_subtree
	}
}