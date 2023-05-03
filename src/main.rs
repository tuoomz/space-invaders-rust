use std::{error::Error, io, time::{Duration, Instant}, sync::mpsc, thread};

use crossterm::{terminal::{EnterAlternateScreen, self}, ExecutableCommand, cursor::{Show, Hide}, event::{KeyCode, Event, self}};
use invaders::{frame::{self, Frame, new_frame, Drawable}, render, player::Player, invader::{Invaders, Invader}};
use rusty_audio::Audio;

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    for item in &["explode", "lose", "move", "pew", "startup", "win"] {
        audio.add(item, &format!("sounds/{}.wav", item));
    }
    audio.play("startup"); 

    let mut stdout= io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;


    let (render_tx, render_rx): (mpsc::Sender<Frame>, mpsc::Receiver<Frame>)  = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        loop {
            let curr_frame = match render_rx.recv(){
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });



    let mut player = Player::new();
    let mut invaders  = Invaders::new();
    let mut instant = Instant::now();

    'gameloop: loop {
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();
     
        
        while event::poll(Duration::default())? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Left => {
                        player.move_left();
                        audio.play("move");
                    }
                    KeyCode::Right => {
                        player.move_right();
                        audio.play("move");
                    }
                    KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    KeyCode::Char(' ') | KeyCode::Enter => {
                       if player.shoot() {
                           audio.play("pew");
                       }
                    }
                    _ => {}
                }
            }
        }
        //Updates
        player.update(delta);
        if invaders.update(delta){
            audio.play("move");
        }

        if player.detect_hits(&mut invaders) {
            audio.play("explode");
        }

        // Draw
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        // Win or loose
        if invaders.all_killed() {
            audio.play("win");
            break 'gameloop;
        }
        else if invaders.reached_botton() {
            audio.play("lose");
            break 'gameloop;
        }
    }

    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();

    Ok(())
}
