pub trait Emulator {
    fn print(&mut self, input: char);
    fn execute(&mut self, input: char);
    fn csi_dispatch(&mut self, input: char, interims: &[char], params: &[u16]);
    fn esc_dispatch(&mut self, input: char, interims: &[char]);
}

#[derive(Debug)]
enum Action {
    None,
    Print,
    Execute,
    CsiDispatch,
    EscDispatch,
    Collect,
    Param,
    Clear,
    Hook,
    Put,
    Unhook,
    OscStart,
    OscPut,
    OscEnd,
}

#[derive(Debug, Default)]
enum State {
    #[default]
    Ground,
    Esc,
    EscInterim,
    CsiEntry,
    CsiIgnore,
    CsiParam,
    CsiInterim,
    DcsEntry,
    DcsInterim,
    DcsIgnore,
    DcsParam,
    DcsPassthrough,
    SosPmApcString,
    OscString,
}

impl State {
    fn enter_action(&self) -> Action {
        match self {
            Self::Esc => Action::Clear,
            Self::CsiEntry => Action::Clear,
            Self::DcsEntry => Action::Clear,
            Self::OscString => Action::OscStart,
            Self::DcsPassthrough => Action::Hook,
            // No constant entry events for all other states
            _ => Action::None,
        }
    }

    fn exit_action(&self) -> Action {
        match self {
            Self::OscString => Action::OscEnd,
            Self::DcsPassthrough => Action::Unhook,
            // No constant exit events for all other states
            _ => Action::None,
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    state: State,
    params: Vec<u16>,
    interims: Vec<char>,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            state: State::default(),
            params: vec![0],
            interims: vec![],
        }
    }
}

// https://github.com/alacritty/vte/blob/master/src/lib.rs
// https://www.vt100.net/emu/dec_ansi_parser
// https://github.com/haberman/vtparse/blob/master/vtparse_tables.rb
impl Parser {
    pub fn feed_str<T: Emulator>(&mut self, input: &str, terminal: &mut T) {
        for ch in input.chars() {
            self.feed(ch, terminal);
        }
    }

    fn feed<T: Emulator>(&mut self, input: char, terminal: &mut T) {
        match self.get_state_change(input) {
            (Some(state), action) => {
                self.perform_action(input, terminal, self.state.exit_action());
                self.perform_action(input, terminal, action);
                self.perform_action(input, terminal, state.enter_action());
                self.state = state;
            }
            (None, action) => {
                self.perform_action(input, terminal, action);
            }
        };
    }

    fn get_state_change(&self, input: char) -> (Option<State>, Action) {
        match (&self.state, input) {
            // |=================================|
            // | anywhere transitions            |
            // |=================================|
            // Ground exclude: '\u{91}'..='\u{97}'|'\u{99}'|'\u{9a}'
            // Others exclude: '\u{90}'|'\u{98}'|'\u{9b}'|'\u{9d}'|'\u{9e}'|'\u{9f}'
            (_, '\u{18}') => (Some(State::Ground), Action::Execute),
            (_, '\u{1a}') => (Some(State::Ground), Action::Execute),
            (_, '\u{80}'..='\u{8f}') => (Some(State::Ground), Action::Execute),
            (_, '\u{9c}') => (Some(State::Ground), Action::None),
            (_, '\u{1b}') => (Some(State::Esc), Action::None),

            // |=================================|
            // | ground events                   |
            // |=================================|
            (State::Ground, '\u{00}'..='\u{17}') => (None, Action::Execute),
            (State::Ground, '\u{19}') => (None, Action::Execute),
            (State::Ground, '\u{1c}'..='\u{1f}') => (None, Action::Execute),
            (State::Ground, '\u{20}'..='\u{7f}') => (None, Action::Print),
            // Handle > 1 byte unicode characters
            (State::Ground, '\u{a0}'..='\u{10ffff}') => (None, Action::Print),

            // |=================================|
            // | escape events                   |
            // |=================================|
            (State::Esc, '\u{00}'..='\u{17}') => (None, Action::Execute),
            (State::Esc, '\u{19}') => (None, Action::Execute),
            (State::Esc, '\u{1c}'..='\u{1f}') => (None, Action::Execute),
            (State::Esc, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | escape transitions              |
            // |=================================|
            (State::Esc, '\u{20}'..='\u{2f}') => (Some(State::EscInterim), Action::Collect),
            (State::Esc, '\u{30}'..='\u{4f}') => (Some(State::Ground), Action::EscDispatch),
            (State::Esc, '\u{51}'..='\u{57}') => (Some(State::Ground), Action::EscDispatch),
            (State::Esc, '\u{59}') => (Some(State::Ground), Action::EscDispatch),
            (State::Esc, '\u{5a}') => (Some(State::Ground), Action::EscDispatch),
            (State::Esc, '\u{5c}') => (Some(State::Ground), Action::EscDispatch),
            (State::Esc, '\u{60}'..='\u{7e}') => (Some(State::Ground), Action::EscDispatch),
            (State::Esc, '\u{50}') => (Some(State::DcsEntry), Action::None),
            (State::Esc, '\u{58}') => (Some(State::SosPmApcString), Action::None),
            (State::Esc, '\u{5e}') => (Some(State::SosPmApcString), Action::None),
            (State::Esc, '\u{5f}') => (Some(State::SosPmApcString), Action::None),
            (State::Esc, '\u{5b}') => (Some(State::CsiEntry), Action::None),
            (State::Esc, '\u{5d}') => (Some(State::OscString), Action::None),

            // |=================================|
            // | escape intermediate events      |
            // |=================================|
            (State::EscInterim, '\u{00}'..='\u{17}') => (None, Action::Execute),
            (State::EscInterim, '\u{19}') => (None, Action::Execute),
            (State::EscInterim, '\u{1c}'..='\u{1f}') => (None, Action::Execute),
            (State::EscInterim, '\u{20}'..='\u{2f}') => (None, Action::Collect),
            (State::EscInterim, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | escape intermediate transitions |
            // |=================================|
            (State::EscInterim, '\u{30}'..='\u{7e}') => (Some(State::Ground), Action::EscDispatch),

            // |=================================|
            // | csi entry events                |
            // |=================================|
            (State::CsiEntry, '\u{00}'..='\u{17}') => (None, Action::Execute),
            (State::CsiEntry, '\u{19}') => (None, Action::Execute),
            (State::CsiEntry, '\u{1c}'..='\u{1f}') => (None, Action::Execute),
            (State::CsiEntry, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | csi entry transitions           |
            // |=================================|
            (State::CsiEntry, '\u{20}'..='\u{2f}') => (Some(State::CsiInterim), Action::Collect),
            (State::CsiEntry, '\u{30}'..='\u{39}') => (Some(State::CsiParam), Action::Param),
            (State::CsiEntry, '\u{3b}') => (Some(State::CsiParam), Action::Param),
            (State::CsiEntry, '\u{3c}'..='\u{3f}') => (Some(State::CsiParam), Action::Collect),
            (State::CsiEntry, '\u{3a}') => (Some(State::CsiIgnore), Action::None),
            (State::CsiEntry, '\u{40}'..='\u{7e}') => (Some(State::Ground), Action::CsiDispatch),

            // |=================================|
            // | csi ignore events               |
            // |=================================|
            (State::CsiIgnore, '\u{00}'..='\u{17}') => (None, Action::Execute),
            (State::CsiIgnore, '\u{19}') => (None, Action::Execute),
            (State::CsiIgnore, '\u{1c}'..='\u{1f}') => (None, Action::Execute),
            (State::CsiIgnore, '\u{20}'..='\u{3f}') => (None, Action::None),
            (State::CsiIgnore, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | csi ignore transitions          |
            // |=================================|
            (State::CsiIgnore, '\u{40}'..='\u{7e}') => (Some(State::Ground), Action::None),

            // |=================================|
            // | csi param events                |
            // |=================================|
            (State::CsiParam, '\u{00}'..='\u{17}') => (None, Action::Execute),
            (State::CsiParam, '\u{19}') => (None, Action::Execute),
            (State::CsiParam, '\u{1c}'..='\u{1f}') => (None, Action::Execute),
            (State::CsiParam, '\u{30}'..='\u{39}') => (None, Action::Param),
            (State::CsiParam, '\u{3b}') => (None, Action::Param),
            (State::CsiParam, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | csi param transitions           |
            // |=================================|
            (State::CsiParam, '\u{20}'..='\u{2f}') => (Some(State::CsiInterim), Action::Collect),
            (State::CsiParam, '\u{3a}') => (Some(State::CsiIgnore), Action::None),
            (State::CsiParam, '\u{3c}'..='\u{3f}') => (Some(State::CsiIgnore), Action::None),
            (State::CsiParam, '\u{40}'..='\u{7e}') => (Some(State::Ground), Action::CsiDispatch),

            // |=================================|
            // | csi intermediate events         |
            // |=================================|
            (State::CsiInterim, '\u{00}'..='\u{17}') => (None, Action::Execute),
            (State::CsiInterim, '\u{19}') => (None, Action::Execute),
            (State::CsiInterim, '\u{1c}'..='\u{1f}') => (None, Action::Execute),
            (State::CsiInterim, '\u{20}'..='\u{2f}') => (None, Action::Collect),
            (State::CsiInterim, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | csi intermediate transitions    |
            // |=================================|
            (State::CsiInterim, '\u{30}'..='\u{3f}') => (Some(State::CsiIgnore), Action::None),
            (State::CsiInterim, '\u{40}'..='\u{7e}') => (Some(State::Ground), Action::CsiDispatch),

            // |=================================|
            // | dcs entry events                |
            // |=================================|
            (State::DcsEntry, '\u{00}'..='\u{17}') => (None, Action::None),
            (State::DcsEntry, '\u{19}') => (None, Action::None),
            (State::DcsEntry, '\u{1c}'..='\u{1f}') => (None, Action::None),
            (State::DcsEntry, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | dcs entry transitions           |
            // |=================================|
            (State::DcsEntry, '\u{20}'..='\u{2f}') => (Some(State::DcsInterim), Action::Collect),
            (State::DcsEntry, '\u{30}'..='\u{39}') => (Some(State::DcsParam), Action::Param),
            (State::DcsEntry, '\u{3b}') => (Some(State::DcsParam), Action::Param),
            (State::DcsEntry, '\u{3a}') => (Some(State::DcsIgnore), Action::None),
            (State::DcsEntry, '\u{3c}'..='\u{3f}') => (Some(State::DcsParam), Action::Collect),
            (State::DcsEntry, '\u{40}'..='\u{7e}') => (Some(State::DcsPassthrough), Action::None),

            // |=================================|
            // | dcs intermediate events         |
            // |=================================|
            (State::DcsInterim, '\u{00}'..='\u{17}') => (None, Action::None),
            (State::DcsInterim, '\u{19}') => (None, Action::None),
            (State::DcsInterim, '\u{1c}'..='\u{1f}') => (None, Action::None),
            (State::DcsInterim, '\u{20}'..='\u{2f}') => (None, Action::Collect),
            (State::DcsInterim, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | dcs intermediate transitions    |
            // |=================================|
            (State::DcsInterim, '\u{30}'..='\u{3f}') => (Some(State::DcsIgnore), Action::None),
            (State::DcsInterim, '\u{40}'..='\u{7e}') => (Some(State::DcsPassthrough), Action::None),

            // |=================================|
            // | dcs ignore events               |
            // |=================================|
            (State::DcsIgnore, '\u{00}'..='\u{17}') => (None, Action::None),
            (State::DcsIgnore, '\u{19}') => (None, Action::None),
            (State::DcsIgnore, '\u{1c}'..='\u{1f}') => (None, Action::None),
            (State::DcsIgnore, '\u{20}'..='\u{7f}') => (None, Action::None),

            // |=================================|
            // | dcs param events                |
            // |=================================|
            (State::DcsParam, '\u{00}'..='\u{17}') => (None, Action::None),
            (State::DcsParam, '\u{19}') => (None, Action::None),
            (State::DcsParam, '\u{1c}'..='\u{1f}') => (None, Action::None),
            (State::DcsParam, '\u{30}'..='\u{39}') => (None, Action::Param),
            (State::DcsParam, '\u{3b}') => (None, Action::Param),
            (State::DcsParam, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | dcs param transitions           |
            // |=================================|
            (State::DcsParam, '\u{20}'..='\u{2f}') => (Some(State::DcsInterim), Action::Collect),
            (State::DcsParam, '\u{3a}') => (Some(State::DcsIgnore), Action::None),
            (State::DcsParam, '\u{3c}'..='\u{3f}') => (Some(State::DcsIgnore), Action::None),
            (State::DcsParam, '\u{40}'..='\u{7e}') => (Some(State::DcsPassthrough), Action::None),

            // |=================================|
            // | dcs passthrough events          |
            // |=================================|
            (State::DcsPassthrough, '\u{00}'..='\u{17}') => (None, Action::Put),
            (State::DcsPassthrough, '\u{19}') => (None, Action::Put),
            (State::DcsPassthrough, '\u{1c}'..='\u{1f}') => (None, Action::Put),
            (State::DcsPassthrough, '\u{20}'..='\u{7e}') => (None, Action::Put),
            (State::DcsPassthrough, '\u{7f}') => (None, Action::None),

            // |=================================|
            // | sos pm apc string events        |
            // |=================================|
            (State::SosPmApcString, '\u{00}'..='\u{17}') => (None, Action::None),
            (State::SosPmApcString, '\u{19}') => (None, Action::None),
            (State::SosPmApcString, '\u{1c}'..='\u{1f}') => (None, Action::None),
            (State::SosPmApcString, '\u{20}'..='\u{7f}') => (None, Action::None),

            // |=================================|
            // | osc string events               |
            // |=================================|
            (State::OscString, '\u{00}'..='\u{06}') => (None, Action::None),
            (State::OscString, '\u{08}'..='\u{17}') => (None, Action::None),
            (State::OscString, '\u{19}') => (None, Action::None),
            (State::OscString, '\u{1c}'..='\u{1f}') => (None, Action::None),
            (State::OscString, '\u{20}'..='\u{7f}') => (None, Action::OscPut),

            // |=================================|
            // | osc string transitions          |
            // |=================================|
            // Support osc string termination with 0x07
            (State::OscString, '\u{07}') => (Some(State::Ground), Action::None),

            _ => panic!("Parser: {:?} | {:x}", self.state, input as u32),
        }
    }

    fn perform_action<T: Emulator>(&mut self, input: char, terminal: &mut T, action: Action) {
        match action {
            Action::None => (),
            Action::Print => terminal.print(input),
            Action::Execute => terminal.execute(input),
            Action::CsiDispatch => terminal.csi_dispatch(input, &self.interims, &self.params),
            Action::EscDispatch => terminal.esc_dispatch(input, &self.interims),
            Action::Collect => self.interims.push(input),
            Action::Param => match input {
                ';' => self.params.push(0),
                '0'..='9' => {
                    let p = self.params.last_mut().unwrap();
                    *p = (10 * (*p)) + input.to_digit(10).unwrap() as u16;
                }
                _ => panic!("Unhandled param: {:?}", input),
            },
            Action::Clear => {
                self.params = vec![0];
                self.interims = vec![];
            }
            // (unhandled)
            Action::Hook => (),
            Action::Put => (),
            Action::Unhook => (),
            Action::OscStart => (),
            Action::OscPut => (),
            Action::OscEnd => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum Event {
        Print(char),
        Execute(char),
        Csi(char, Vec<char>, Vec<u16>),
        Esc(char, Vec<char>),
    }

    #[derive(Debug, Default)]
    struct TestEmulator {
        events: Vec<Event>,
    }

    impl Emulator for TestEmulator {
        fn print(&mut self, input: char) {
            self.events.push(Event::Print(input));
        }

        fn execute(&mut self, input: char) {
            self.events.push(Event::Execute(input));
        }

        fn csi_dispatch(&mut self, input: char, interims: &[char], params: &[u16]) {
            self.events
                .push(Event::Csi(input, interims.to_vec(), params.to_vec()));
        }

        fn esc_dispatch(&mut self, input: char, interims: &[char]) {
            self.events.push(Event::Esc(input, interims.to_vec()));
        }
    }

    #[test]
    fn print() {
        run(
            "Print exact characters to buffer",
            "ab",
            &[Event::Print('a'), Event::Print('b')],
        );
    }

    #[test]
    fn execute() {
        run("Execute a backspace", "\x08", &[Event::Execute('\u{08}')]);
    }

    #[test]
    fn csi() {
        run(
            "Move cursor to start",
            "\x1b[H",
            &[Event::Csi('H', vec![], vec![0])],
        );
        run(
            "Cursor visibility",
            "\x1b[34h",
            &[Event::Csi('h', vec![], vec![34])],
        );
        run(
            "Report keyboard mode",
            "\x1b[?u",
            &[Event::Csi('u', vec!['?'], vec![0])],
        );
        run(
            "Switch to alternate buffer",
            "\x1b[?1049h",
            &[Event::Csi('h', vec!['?'], vec![1049])],
        );
        run(
            "Report private mode",
            "\x1b[?2026$p",
            &[Event::Csi('p', vec!['?', '$'], vec![2026])],
        );
        //run(
        //    "Report private mode",
        //    "\x1b]112\x07",
        //    &[Event::Csi('p', vec!['?', '$'], vec![2026])],
        //);
    }

    #[test]
    fn esc() {
        run(
            "Set ascii character set",
            "\x1b(A",
            &[Event::Esc('A', vec!['('])],
        );
        run(
            "Set keypad application mode",
            "\x1b=",
            &[Event::Esc('=', vec![])],
        );
    }

    #[test]
    fn color() {
        run(
            "Set foreground RGB",
            "\x1b[38;2;200;200;200mHi\x1b[0m",
            &[
                Event::Csi('m', vec![], vec![38, 2, 200, 200, 200]),
                Event::Print('H'),
                Event::Print('i'),
                Event::Csi('m', vec![], vec![0]),
            ],
        );
    }

    #[test]
    fn todo() {
        run("TEMP", "\x1b]11;?\x07", &[]);
        //run("temp", "\x1b]112\x07\x1b[2 q\x1b]112\x07\x1b[2 q\x1b[?1002h\x1b[?1006h\x1b[38;2;0;0;0m\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[K\n\x1b[J\x1b[H\x1b[34h\x1b[?25h", &[]);
        //run("temp", "\x1b[?25l\x1b[0;1m\x0f\x1b[38;2;224;222;244m\x1b[48;2;25;23;36m1   \x1b[0;1m\x0f\x1b[38;2;196;167;231m\x1b[48;2;38;35;58m#\x1b[m\x0f\x1b[38;2;224;222;244m\x1b[48;2;38;35;58m \x1b[0;1m\x0f\x1b[38;2;196;167;231m\x1b[48;2;38;35;58mNote\x1b[m\x0f\x1b[38;2;224;222;244m\x1b[48;2;38;35;58m                                             \r\n\x1b[m\x0f\x1b[38;2;110;106;134m\x1b[48;2;25;23;36m  1 \x1b[m\x0f\x1b[38;2;224;222;244m\x1b[48;2;25;23;36m                                                   \r\n\x1b[m\x0f\x1b[38;2;110;106;134m\x1b[48;2;25;23;36m  2 \x1b[m\x0f\x1b[38;2;144;140;170m\x1b[48;2;25;23;36m> [\x1b[m\x0f\x1b[38;2;156;207;216m\x1b[48;2;25;23;36m!NOTE\x1b[m\x0f\x1b[38;2;144;140;170m\x1b[48;2;25;23;36m]\x1b[m\x0f\x1b[38;2;224;222;244m\x1b[48;2;25;23;36m                                          \r\n\x1b[m\x0f\x1b[38;2;110;106;134m\x1b[48;2;25;23;36m  3 \x1b[m\x0f\x1b[38;2;144;140;170m\x1b[48;2;25;23;36m> A regular note\x1b[m\x0f\x1b[38;2;224;222;244m\x1b[48;2;25;23;36m                                   \r\n\x1b[m\x0f\x1b[38;2;110;106;134m\x1b[48;2;25;23;36m  4 \x1b[m\x0f\x1b[38;2;224;222;244m\x1b[48;2;25;23;36m                                                   \r\n\x1b[m\x0f\x1b[38;2;110;106;134m\x1b[48;2;25;23;36m  5 \x1b[0;1m\x0f\x1b[38;2;196;", &[]);
    }

    fn run(message: &str, input: &str, expected: &[Event]) {
        let mut parser = Parser::default();
        let mut terminal = TestEmulator::default();
        parser.feed_str(input, &mut terminal);
        assert_eq!(terminal.events, expected, "{}", message);
    }
}
