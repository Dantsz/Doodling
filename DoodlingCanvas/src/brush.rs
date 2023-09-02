use crate::{render_state::{State, Vertex, RenderCommands}, utils};

 //TODO: Use unforms instead calculating vertices here
 pub struct Rectangle{
    vertices: [Vertex;6],
    buffer: Option<wgpu::Buffer>,
 }

 impl Rectangle{
    pub fn new(dimensions : [f32;4]) -> Self
    {
        let win = (utils::WINDOW_WIDTH as f32 / 2.0,utils::WINDOW_HEIGHT as f32 / 2.0);
        let dimensions = [dimensions[0] , -dimensions[1], dimensions[2], dimensions[3]];
        let vertices : [Vertex;6]= [
            Vertex{position: [dimensions[0]/win.0 - 1.0,dimensions[1]/win.1 + 1.0]},
            Vertex{position: [dimensions[0]/win.0 - 1.0,(dimensions[1]-dimensions[3])/win.1 + 1.0]},
            Vertex{position: [(dimensions[0]+dimensions[2])/win.0 - 1.0,(dimensions[1]-dimensions[3])/win.1 + 1.0]},
            Vertex{position: [(dimensions[0]+dimensions[2])/win.0 - 1.0,(dimensions[1]-dimensions[3])/win.1  + 1.0]},
            Vertex{position: [(dimensions[0]+dimensions[2])/win.0 - 1.0,dimensions[1]/win.1 + 1.0]},
            Vertex{position: [dimensions[0]/win.0 - 1.0,dimensions[1]/win.1 + 1.0]},
        ];
        Self { vertices,buffer: None }
    }
    pub fn new_cached(render_state: &mut State, dimensions : [f32;4]) -> Self
    {
        let mut rect = Self::new(dimensions);
        rect.buffer = Some(render_state.make_test_buffer(rect.vertices.as_slice()));
        rect
    }
    // Prepares the rectangle for drawing adnd registers the drawing command to the render state
    pub fn draw_to(&mut self, render_state : &mut State,commands: &mut RenderCommands, offset : [f32;2])
    {
        self.buffer =  Some(render_state.make_test_buffer(self.vertices.as_slice()));
        let adjust_offset = [2.0 * offset[0] / (utils::WINDOW_WIDTH as f32 ) ,2.0 * offset[1] / (utils::WINDOW_HEIGHT as f32 ) ];
        render_state.draw_buffer(commands, self.buffer.as_ref().unwrap(),adjust_offset);
    }
 }