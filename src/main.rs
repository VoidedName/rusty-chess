use sdl2::Sdl;

fn main() -> Result<(), String> {
    let context = sdl2::init()?;

    println!("Hello, world!");

    Ok(())
}
