extern crate glutin;
extern crate glutin_frontend;

fn main() {
    let mut frontend = glutin_frontend::Frontend::new();
    let mut running = true;
    loop {
        frontend.poll_glutin_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => {
                    running = false;
                }
                _ => (),
            },
            _ => (),
        });
        if !running {
            break;
        }
        frontend.render();
    }
}
