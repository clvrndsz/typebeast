use crossterm::event::{KeyCode, KeyEvent};
use random_word::Lang;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::{Line,Span},
    widgets::{Wrap, Widget, Paragraph},
};
use std::{io, sync::mpsc, thread};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    // Create the channel via which the events will be sent to the main app.
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    // Thread to listen for input events.
    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let mut app = App {
        exit: false,
        global_index: 0,
        ref_text: App::get_words(25),
        
        next_char: ' ',
        color: Color::Gray,
    };

    // App runs on the main thread.
    let app_result = app.run(&mut terminal, event_rx);

    // Note: If your threads need clean-up (i.e. the computation thread),
    // you should communicate to them that the app wants to shut down.
    // This is not required here, as our threads don't use resources.
    ratatui::restore();
    app_result
}

// Events that can be sent to the main thread.
enum Event {
    Input(crossterm::event::KeyEvent), // crossterm key input event
}


pub struct App <'a> {
    exit: bool,
    global_index: usize,
    ref_text: Line<'a>,
    out_text: Line<'a>,
    next_char: char,
    color: Color,
}

/// Block, waiting for input events from the user.
fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx
                .send(Event::Input(key_event))
                .unwrap(),
            _ => {}
        }
    }
}



impl <'a> App <'a> {
    /// Main task to be run continuously
    
    
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn get_words(mut count: u8) -> Line<'a> {
        let mut vector = Line::default();
        while count != 0 {
            let current_word = random_word::get(Lang::En);
            for c in current_word.chars() {
                let holder = Span::styled(c.to_string(), Style::default().fg(Color::Gray));
                vector.push_span(holder);
            }
            let spacer = Span::styled(" ", Style::default());
            vector.push_span(spacer);
            count -= 1;
        }
        vector

    }

    fn update_text(&self, prev_character: char, color: Color)-> Line<'a> {
    // TODO: loop through the entire line and copy over values one by one, 
    // in the place of the global index location, handle the logic based on 
    // the key that was entered, and paste the rest of the original line 
    // after that, and return
    // NEED TO CREATE PREVIOUS CHARACTER FUNCTIONALITY : neeeded to make sure that the backspace
    // works
    let content = 'i';
    let new_span: Span<'a> = Span::styled(content.to_string(), color);
    let mut new_text: Line<'a> = Line::default();
    for i in self.text.clone() {
        let mut index = 0;
        if self.global_index == index {
            new_text.push_span(new_span.clone());
        } else {
            new_text.push_span(i);
        }
        index += 1;
        
    }

    new_text
    }


    /// Render `self`, as we implemented the Widget trait for &App
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn get_next_char(&self) -> char {
        let result = self.ref_text.iter()
            .nth(self.global_index).unwrap()
            .content.chars().last().unwrap();

        result
    }

    /// Actions that should be taken when a key event comes in. Append Char here to display string.
    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {

        match key_event.code {
            KeyCode::Backspace => {
                // Remove last output char and move index back if possible
                if self.global_index > 0 {
                    self.out_text = self.update_text('s', self.color);
                    self.global_index -= 1; 
                    self.next_char = self.get_next_char(); 

                }
            }

            KeyCode::Enter => {
                // Exit the app on Enter.
                self.exit = true;
            }

            KeyCode::Char(c) => {
                // Normal typing
                let correct = self.next_char == c;
                self.color = if correct { Color::Green } else { Color::Red };
                self.out_text = self.update_text('s', self.color);
                self.global_index += 1;
                self.next_char = self.get_next_char();
            }
            _ => {
                self.exit = false;
            }
        }
        Ok(())
    }
}

impl <'a> Widget for &App <'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split the screen vertically into horizontal zones. 
        let vertical_layout = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ]);
        let [title_area,input_area] = vertical_layout.areas(area);

        Line::from("typebeast").bold().centered().render(title_area, buf);

        // Render the words to type
        Paragraph::new(self.ref_text.clone())
            .wrap(Wrap {trim : true})
            .centered()
            .render(input_area, buf); 

    }
}
