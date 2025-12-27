use pipewire::{main_loop::MainLoopBox, context::ContextBox};

pub fn get_audio_registry() -> Result<(), Box<dyn std::error::Error>> {
    let mainloop = MainLoopBox::new(None)?;
    let context = ContextBox::new(&mainloop.loop_(), None)?;
    let core = context.connect(None)?;
    let registry = core.get_registry()?;
    println!("{:?}", registry);

    Ok(())
}