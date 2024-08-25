use std::{cell::RefCell, rc::Rc};

use crate::{debug::widget::debug::DebugItem, graphics::shapes::Shape};

pub trait Scene {
    // ...

    fn debug_item(&self) -> Vec<Rc<RefCell<dyn DebugItem>>> {
        Vec::new()
    }

    fn update(&mut self, dt: std::time::Duration);

    /// Returns the shapes that are part of the scene
    /// This is used to load the shapes into the GPU
    /// and render them
    ///
    /// # Returns
    ///
    /// A vector of shapes that have been updated
    fn shapes(&self) -> Vec<Rc<RefCell<Shape>>> {
        Vec::new()
    }
}
