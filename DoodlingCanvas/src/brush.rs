use crate::{render_state::{State, Vertex, RenderCommands}, utils};

 //TODO: Use unforms instead calculating vertices here
 pub struct Rectangle{
    vertices: [Vertex;6],
    buffer: wgpu::Buffer,
 }

 impl Rectangle{
    pub fn new(render_state: &mut State, dimensions : [f32;4]) -> Self
    {
        let rect: [Vertex; 6] = Self::dimensions_to_vertices(dimensions);
        let inner_buff = render_state.make_test_buffer(rect.as_slice());
        Self { vertices: rect, buffer: inner_buff }
    }
    fn dimensions_to_vertices(dimensions : [f32;4]) -> [Vertex;6]
    {
        let win = (utils::WINDOW_WIDTH as f32 / 2.0,utils::WINDOW_HEIGHT as f32 / 2.0);
        let dimensions = [dimensions[0] , -dimensions[1], dimensions[2], dimensions[3]];
        [
            Vertex{position: [dimensions[0]/win.0 - 1.0,dimensions[1]/win.1 + 1.0]},
            Vertex{position: [dimensions[0]/win.0 - 1.0,(dimensions[1]-dimensions[3])/win.1 + 1.0]},
            Vertex{position: [(dimensions[0]+dimensions[2])/win.0 - 1.0,(dimensions[1]-dimensions[3])/win.1 + 1.0]},
            Vertex{position: [(dimensions[0]+dimensions[2])/win.0 - 1.0,(dimensions[1]-dimensions[3])/win.1  + 1.0]},
            Vertex{position: [(dimensions[0]+dimensions[2])/win.0 - 1.0,dimensions[1]/win.1 + 1.0]},
            Vertex{position: [dimensions[0]/win.0 - 1.0,dimensions[1]/win.1 + 1.0]},
        ]
    }
    // Prepares the rectangle for drawing adnd registers the drawing command to the render state
    pub fn draw_to(&mut self, render_state : &mut State,commands: &mut RenderCommands, offset : [f32;2])
    {
        self.buffer =  render_state.make_test_buffer(self.vertices.as_slice());
        let adjust_offset = [2.0 * offset[0] / (utils::WINDOW_WIDTH as f32 ) ,2.0 * offset[1] / (utils::WINDOW_HEIGHT as f32 ) ];
        render_state.draw_buffer(commands, &self.buffer,adjust_offset);
    }
 }