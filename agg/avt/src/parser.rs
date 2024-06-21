use crate::terminal::Terminal;

#[derive(Debug)]
enum Action {
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
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiIgnore,
    CsiParam,
    CsiIntermediate,
    DcsEntry,
    DcsIntermediate,
    DcsIgnore,
    DcsParam,
    DcsPassthrough,
    SosPmApcString,
    OscString,
}

impl State {
    fn enter_action(&self) -> Option<Action> {
        match self {
            State::Escape => Some(Action::Clear),
            State::CsiEntry => Some(Action::Clear),
            State::DcsEntry => Some(Action::Clear),
            State::OscString => Some(Action::OscStart),
            State::DcsPassthrough => Some(Action::Hook),
            // No constant entry events for all other states
            _ => None,
        }
    }

    fn exit_action(&self) -> Option<Action> {
        match self {
            Self::OscString => Some(Action::OscEnd),
            Self::DcsPassthrough => Some(Action::Unhook),
            // No constant exit events for all other states
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    state: State,
    params: Vec<u16>,
    intermediates: Vec<char>,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            state: State::default(),
            params: vec![0],
            intermediates: vec![],
        }
    }
}

// https://www.vt100.net/emu/dec_ansi_parser
// https://github.com/haberman/vtparse/blob/master/vtparse_tables.rb
impl Parser {
    pub fn feed_str(&mut self, input: &str, terminal: &mut Terminal) {
        for ch in input.chars() {
            self.feed(ch, terminal);
        }
    }

    fn feed(&mut self, input: char, terminal: &mut Terminal) {
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

    fn get_state_change(&self, input: char) -> (Option<State>, Option<Action>) {
        match (&self.state, input) {
            // |=================================|
            // | anywhere transitions            |
            // |=================================|
            // Excluded cases: 80..=8f|91..=97|99|9a|9c|98|9e|9f|90|9d|9b
            (_, '\u{18}') | (_, '\u{1a}') => (Some(State::Ground), Some(Action::Execute)),
            (_, '\u{1b}') => (Some(State::Escape), None),

            // |=================================|
            // | ground events                   |
            // |=================================|
            (State::Ground, '\u{00}'..='\u{17}')
            | (State::Ground, '\u{19}')
            | (State::Ground, '\u{1c}'..='\u{1f}') => (None, Some(Action::Execute)),
            (State::Ground, '\u{20}'..='\u{7f}') => (None, Some(Action::Print)),
            // Handle > 1 byte unicode characters
            (State::Ground, '\u{a0}'..='\u{10ffff}') => (None, Some(Action::Print)),

            // |=================================|
            // | escape events                   |
            // |=================================|
            (State::Escape, '\u{00}'..='\u{17}')
            | (State::Escape, '\u{19}')
            | (State::Escape, '\u{1c}'..='\u{1f}') => (None, Some(Action::Execute)),
            (State::Escape, '\u{7f}') => (None, None),

            // |=================================|
            // | escape transitions              |
            // |=================================|
            (State::Escape, '\u{20}'..='\u{2f}') => {
                (Some(State::EscapeIntermediate), Some(Action::Collect))
            }
            (State::Escape, '\u{30}'..='\u{4f}')
            | (State::Escape, '\u{51}'..='\u{57}')
            | (State::Escape, '\u{59}')
            | (State::Escape, '\u{5a}')
            | (State::Escape, '\u{5c}')
            | (State::Escape, '\u{60}'..='\u{7e}') => {
                (Some(State::Ground), Some(Action::EscDispatch))
            }
            (State::Escape, '\u{5b}') => (Some(State::CsiEntry), None),
            (State::Escape, '\u{5d}') => (Some(State::OscString), None),
            (State::Escape, '\u{50}') => (Some(State::DcsEntry), None),
            (State::Escape, '\u{58}') | (State::Escape, '\u{5e}') | (State::Escape, '\u{5f}') => {
                (Some(State::SosPmApcString), None)
            }

            // |=================================|
            // | escape intermediate events      |
            // |=================================|
            (State::EscapeIntermediate, '\u{00}'..='\u{17}')
            | (State::EscapeIntermediate, '\u{19}')
            | (State::EscapeIntermediate, '\u{1c}'..='\u{1f}') => (None, Some(Action::Execute)),
            (State::EscapeIntermediate, '\u{20}'..='\u{2f}') => (None, Some(Action::Collect)),

            (State::EscapeIntermediate, '\u{7f}') => (None, None),

            // |=================================|
            // | escape intermediate transitions |
            // |=================================|
            (State::EscapeIntermediate, '\u{30}'..='\u{7e}') => {
                (Some(State::Ground), Some(Action::EscDispatch))
            }

            // |=================================|
            // | csi entry events                |
            // |=================================|
            (State::CsiEntry, '\u{00}'..='\u{17}')
            | (State::CsiEntry, '\u{19}')
            | (State::CsiEntry, '\u{1c}'..='\u{1f}') => (None, Some(Action::Execute)),
            (State::CsiEntry, '\u{7f}') => (None, None),

            // |=================================|
            // | csi entry transitions           |
            // |=================================|
            (State::CsiEntry, '\u{20}'..='\u{2f}') => {
                (Some(State::CsiIntermediate), Some(Action::Collect))
            }
            (State::CsiEntry, '\u{3a}') => (Some(State::CsiIgnore), None),
            (State::CsiEntry, '\u{30}'..='\u{39}') | (State::CsiEntry, '\u{3b}') => {
                (Some(State::CsiParam), Some(Action::Param))
            }
            (State::CsiEntry, '\u{3c}'..='\u{3f}') => {
                (Some(State::CsiParam), Some(Action::Collect))
            }
            (State::CsiEntry, '\u{40}'..='\u{7e}') => {
                (Some(State::Ground), Some(Action::CsiDispatch))
            }

            // |=================================|
            // | csi ignore events               |
            // |=================================|
            (State::CsiIgnore, '\u{00}'..='\u{17}')
            | (State::CsiIgnore, '\u{19}')
            | (State::CsiIgnore, '\u{1c}'..='\u{1f}') => (None, Some(Action::Execute)),
            (State::CsiIgnore, '\u{20}'..='\u{3f}') | (State::CsiIgnore, '\u{7f}') => (None, None),

            // |=================================|
            // | csi ignore transitions          |
            // |=================================|
            (State::CsiIgnore, '\u{40}'..='\u{7e}') => (Some(State::Ground), None),

            // |=================================|
            // | csi param events                |
            // |=================================|
            (State::CsiParam, '\u{00}'..='\u{17}')
            | (State::CsiParam, '\u{19}')
            | (State::CsiParam, '\u{1c}'..='\u{1f}') => (None, Some(Action::Execute)),
            (State::CsiParam, '\u{30}'..='\u{39}') | (State::CsiParam, '\u{3b}') => {
                (None, Some(Action::Param))
            }
            (State::CsiParam, '\u{7f}') => (None, None),

            // |=================================|
            // | csi param transitions           |
            // |=================================|
            (State::CsiParam, '\u{3a}') | (State::CsiParam, '\u{3c}'..='\u{3f}') => {
                (Some(State::CsiIgnore), None)
            }
            (State::CsiParam, '\u{20}'..='\u{2f}') => {
                (Some(State::CsiIntermediate), Some(Action::Collect))
            }
            (State::CsiParam, '\u{40}'..='\u{7e}') => {
                (Some(State::Ground), Some(Action::CsiDispatch))
            }

            // |=================================|
            // | csi intermediate events         |
            // |=================================|
            (State::CsiIntermediate, '\u{00}'..='\u{17}')
            | (State::CsiIntermediate, '\u{19}')
            | (State::CsiIntermediate, '\u{1c}'..='\u{1f}') => (None, Some(Action::Execute)),
            (State::CsiIntermediate, '\u{20}'..='\u{2f}') => (None, Some(Action::Collect)),
            (State::CsiIntermediate, '\u{7f}') => (None, None),

            // |=================================|
            // | csi intermediate transitions    |
            // |=================================|
            (State::CsiIntermediate, '\u{30}'..='\u{3f}') => (Some(State::CsiIgnore), None),
            (State::CsiIntermediate, '\u{40}'..='\u{7e}') => {
                (Some(State::Ground), Some(Action::CsiDispatch))
            }

            // |=================================|
            // | dcs entry events                |
            // |=================================|
            (State::DcsEntry, '\u{00}'..='\u{17}')
            | (State::DcsEntry, '\u{19}')
            | (State::DcsEntry, '\u{1c}'..='\u{1f}')
            | (State::DcsEntry, '\u{7f}') => (None, None),

            // |=================================|
            // | dcs entry transitions           |
            // |=================================|
            (State::DcsEntry, '\u{3a}') => (Some(State::DcsIgnore), None),
            (State::DcsEntry, '\u{20}'..='\u{2f}') => {
                (Some(State::DcsIntermediate), Some(Action::Collect))
            }
            (State::DcsEntry, '\u{30}'..='\u{39}') | (State::DcsEntry, '\u{3b}') => {
                (Some(State::DcsParam), Some(Action::Param))
            }
            (State::DcsEntry, '\u{3c}'..='\u{3f}') => {
                (Some(State::DcsParam), Some(Action::Collect))
            }
            (State::DcsEntry, '\u{40}'..='\u{7e}') => (Some(State::DcsPassthrough), None),

            // |=================================|
            // | dcs intermediate events         |
            // |=================================|
            (State::DcsIntermediate, '\u{00}'..='\u{17}')
            | (State::DcsIntermediate, '\u{19}')
            | (State::DcsIntermediate, '\u{1c}'..='\u{1f}') => (None, None),
            (State::DcsIntermediate, '\u{20}'..='\u{2f}') => (None, Some(Action::Collect)),
            (State::DcsIntermediate, '\u{7f}') => (None, None),

            // |=================================|
            // | dcs intermediate transitions    |
            // |=================================|
            (State::DcsIntermediate, '\u{30}'..='\u{3f}') => (Some(State::DcsIgnore), None),
            (State::DcsIntermediate, '\u{40}'..='\u{7e}') => (Some(State::DcsPassthrough), None),

            // |=================================|
            // | dcs ignore events               |
            // |=================================|
            (State::DcsIgnore, '\u{00}'..='\u{17}')
            | (State::DcsIgnore, '\u{19}')
            | (State::DcsIgnore, '\u{1c}'..='\u{1f}')
            | (State::DcsIgnore, '\u{20}'..='\u{7f}') => (None, None),

            // |=================================|
            // | dcs param events                |
            // |=================================|
            (State::DcsParam, '\u{00}'..='\u{17}')
            | (State::DcsParam, '\u{19}')
            | (State::DcsParam, '\u{1c}'..='\u{1f}') => (None, None),
            (State::DcsParam, '\u{30}'..='\u{39}') | (State::DcsParam, '\u{3b}') => {
                (None, Some(Action::Param))
            }
            (State::DcsParam, '\u{7f}') => (None, None),

            // |=================================|
            // | dcs param transitions           |
            // |=================================|
            (State::DcsParam, '\u{3a}') | (State::DcsParam, '\u{3c}'..='\u{3f}') => {
                (Some(State::DcsIgnore), None)
            }
            (State::DcsParam, '\u{20}'..='\u{2f}') => {
                (Some(State::DcsIntermediate), Some(Action::Collect))
            }
            (State::DcsParam, '\u{40}'..='\u{7e}') => (Some(State::DcsPassthrough), None),

            // |=================================|
            // | dcs passthrough events          |
            // |=================================|
            (State::DcsPassthrough, '\u{00}'..='\u{17}')
            | (State::DcsPassthrough, '\u{19}')
            | (State::DcsPassthrough, '\u{1c}'..='\u{1f}')
            | (State::DcsPassthrough, '\u{20}'..='\u{7e}') => (None, Some(Action::Put)),
            (State::DcsPassthrough, '\u{7f}') => (None, None),

            // |=================================|
            // | sos pm apc string events        |
            // |=================================|
            (State::SosPmApcString, '\u{00}'..='\u{17}')
            | (State::SosPmApcString, '\u{19}')
            | (State::SosPmApcString, '\u{1c}'..='\u{1f}')
            | (State::SosPmApcString, '\u{20}'..='\u{7f}') => (None, None),

            // |=================================|
            // | osc string events               |
            // |=================================|
            (State::OscString, '\u{00}'..='\u{17}')
            | (State::OscString, '\u{19}')
            | (State::OscString, '\u{1c}'..='\u{1f}') => (None, None),
            (State::OscString, '\u{20}'..='\u{7f}') => (None, Some(Action::OscPut)),

            _ => panic!(
                "Unhandled state / input pair: {:?} / {:x}",
                self.state, input as u32
            ),
        }
    }

    fn perform_action(&mut self, input: char, terminal: &mut Terminal, action: Option<Action>) {
        if let Some(action) = action {
            match action {
                Action::Print => terminal.print(input),
                Action::Execute => terminal.execute(input),
                Action::CsiDispatch => {
                    terminal.csi_dispatch(&self.params, &self.intermediates, input)
                }
                Action::EscDispatch => terminal.esc_dispatch(&self.intermediates, input),
                Action::Collect => self.intermediates.push(input),
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
                    self.intermediates = vec![];
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
}
