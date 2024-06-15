use crate::charset::Charset;
use crate::terminal::Terminal;

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Ground,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    CsiIgnore,
    DcsEntry,
    DcsParam,
    DcsIntermediate,
    DcsPassthrough,
    DcsIgnore,
    OscString,
    SosPmApcString,
}

#[derive(Debug)]
pub struct Params(Vec<u16>);

impl Params {
    pub fn iter(&self) -> std::slice::Iter<u16> {
        self.0.iter()
    }

    pub fn as_slice(&self) -> &[u16] {
        &self.0[..]
    }

    pub fn get(&self, i: usize, default: usize) -> usize {
        let param = *self.0.get(i).unwrap_or(&0);
        if param == 0 {
            default
        } else {
            param as usize
        }
    }
}

impl Default for Params {
    fn default() -> Self {
        let mut params = Vec::with_capacity(8);
        params.push(0);
        Self(params)
    }
}

#[derive(Debug, Default)]
pub struct Intermediates(Vec<char>);

#[derive(Debug, Default)]
pub struct Parser {
    state: State,
    params: Params,
    intermediates: Intermediates,
}

impl Parser {
    pub fn feed_str(&mut self, input: &str, terminal: &mut Terminal) {
        for ch in input.chars() {
            self.feed(ch, terminal);
        }
    }

    // https://www.vt100.net/emu/dec_ansi_parser
    pub fn feed(&mut self, input: char, terminal: &mut Terminal) {
        let clamped_input = if input >= '\u{a0}' { '\u{41}' } else { input };
        match (&self.state, clamped_input) {
            // anywhere -> ground w/ execute
            (_, '\u{18}')
            | (_, '\u{1a}')
            | (_, '\u{80}'..='\u{8f}')
            | (_, '\u{91}'..='\u{97}')
            | (_, '\u{99}')
            | (_, '\u{9a}') => {
                self.enter(State::Ground);
                self.execute(terminal, input);
            }

            // anywhere -> ground
            // should run osc end for osc string, currently unhandled
            // should run unhook for dcs passthrough, currently unhandled
            (_, '\u{9c}') => self.enter(State::Ground),

            // ground event / execute
            (State::Ground, '\u{00}'..='\u{17}') => self.execute(terminal, input),
            (State::Ground, '\u{19}') => self.execute(terminal, input),
            (State::Ground, '\u{1c}'..='\u{1f}') => self.execute(terminal, input),

            // ground event / print
            (State::Ground, '\u{20}'..='\u{7f}') => terminal.print(input),

            // anywhere -> escape
            (_, '\u{1b}') => self.enter(State::Escape),

            // escape event / execute
            (State::Escape, '\u{00}'..='\u{17}') => self.execute(terminal, input),
            (State::Escape, '\u{19}') => self.execute(terminal, input),
            (State::Escape, '\u{1c}'..='\u{1f}') => self.execute(terminal, input),

            // escape event / ignore
            (State::Escape, '\u{7f}') => (),

            // escape -> escape intermediate w/ collect
            (State::Escape, '\u{20}'..='\u{2f}') => {
                self.enter(State::EscapeIntermediate);
                self.collect(input);
            }

            // escape -> ground w/ esc dispatch
            (State::Escape, '\u{30}'..='\u{4f}')
            | (State::Escape, '\u{51}'..='\u{57}')
            | (State::Escape, '\u{59}')
            | (State::Escape, '\u{5a}')
            | (State::Escape, '\u{5c}')
            | (State::Escape, '\u{60}'..='\u{7e}') => {
                self.enter(State::Ground);
                self.esc_dispatch(terminal, input);
            }

            // escape output transitions
            (State::Escape, '\u{5b}') => self.enter(State::CsiEntry),
            (State::Escape, '\u{5d}') => self.enter(State::OscString),
            (State::Escape, '\u{50}') => self.enter(State::DcsEntry),
            (State::Escape, '\u{58}') | (State::Escape, '\u{5e}') | (State::Escape, '\u{5f}') => {
                self.enter(State::SosPmApcString)
            }

            // escape intermediate event -> execute
            (State::EscapeIntermediate, '\u{00}'..='\u{17}') => self.execute(terminal, input),
            (State::EscapeIntermediate, '\u{19}') => self.execute(terminal, input),
            (State::EscapeIntermediate, '\u{1c}'..='\u{1f}') => self.execute(terminal, input),

            // escape intermediate event -> collect
            (State::EscapeIntermediate, '\u{20}'..='\u{2f}') => self.collect(input),

            // escape intermediate event / ignore
            (State::EscapeIntermediate, '\u{7f}') => (),

            // escape intermediate -> ground w/ esc dispatch
            (State::EscapeIntermediate, '\u{30}'..='\u{7e}') => {
                self.enter(State::Ground);
                self.esc_dispatch(terminal, input);
            }

            // anywhere -> csi entry
            (_, '\u{9b}') => self.enter(State::CsiEntry),

            // csi entry event / execute
            (State::CsiEntry, '\u{00}'..='\u{17}') => self.execute(terminal, input),
            (State::CsiEntry, '\u{19}') => self.execute(terminal, input),
            (State::CsiEntry, '\u{1c}'..='\u{1f}') => self.execute(terminal, input),

            // csi entry event / ignore
            (State::CsiEntry, '\u{7f}') => (),

            // csi entry -> csi param w/ param
            (State::CsiEntry, '\u{30}'..='\u{39}') | (State::CsiEntry, '\u{3b}') => {
                self.enter(State::CsiParam);
                self.param(input);
            }

            // csi entry -> csi param w/ collect
            (State::CsiEntry, '\u{3c}'..='\u{3f}') => {
                self.enter(State::CsiParam);
                self.collect(input);
            }

            // csi entry -> csi ignore
            (State::CsiEntry, '\u{3a}') => self.enter(State::CsiIgnore),

            // csi entry -> csi intermediate w/ collect
            (State::CsiEntry, '\u{20}'..='\u{2f}') => {
                self.enter(State::CsiIntermediate);
                self.collect(input);
            }

            // csi entry -> ground w/ csi dispatch
            (State::CsiEntry, '\u{40}'..='\u{7e}') => {
                self.enter(State::Ground);
                self.csi_dispatch(terminal, input);
            }

            // csi param event / execute
            (State::CsiParam, '\u{00}'..='\u{17}') => self.execute(terminal, input),
            (State::CsiParam, '\u{19}') => self.execute(terminal, input),
            (State::CsiParam, '\u{1c}'..='\u{1f}') => self.execute(terminal, input),

            // csi param event / param
            (State::CsiParam, '\u{30}'..='\u{39}') => self.param(input),
            (State::CsiParam, '\u{3b}') => self.param(input),

            // csi param event / ignore
            (State::CsiParam, '\u{7f}') => (),

            // csi param -> csi ignore
            (State::CsiParam, '\u{3a}') => self.enter(State::CsiIgnore),
            (State::CsiParam, '\u{3c}'..='\u{3f}') => self.enter(State::CsiIgnore),

            // csi param -> csi intermediate w/ collect
            (State::CsiParam, '\u{20}'..='\u{2f}') => {
                self.enter(State::CsiIntermediate);
                self.collect(input);
            }

            // csi param -> ground w/ csi dispatch
            (State::CsiParam, '\u{40}'..='\u{7e}') => {
                self.enter(State::Ground);
                self.csi_dispatch(terminal, input);
            }

            // csi ignore event / execute
            (State::CsiIgnore, '\u{00}'..='\u{17}') => self.execute(terminal, input),
            (State::CsiIgnore, '\u{19}') => self.execute(terminal, input),
            (State::CsiIgnore, '\u{1c}'..='\u{1f}') => self.execute(terminal, input),

            // csi ignore event / ignore
            (State::CsiIgnore, '\u{7f}') => (),

            // csi ignore -> ground
            (State::CsiIgnore, '\u{40}'..='\u{7e}') => self.enter(State::Ground),

            // csi intermediate event / execute
            (State::CsiIntermediate, '\u{00}'..='\u{17}') => self.execute(terminal, input),
            (State::CsiIntermediate, '\u{19}') => self.execute(terminal, input),
            (State::CsiIntermediate, '\u{1c}'..='\u{1f}') => self.execute(terminal, input),

            // csi intermediate event / collect
            (State::CsiIntermediate, '\u{20}'..='\u{2f}') => self.collect(input),

            // csi intermediate event / ignore
            (State::CsiIntermediate, '\u{7f}') => (),

            // csi intermediate -> csi ignore
            (State::CsiIntermediate, '\u{30}'..='\u{3f}') => self.enter(State::CsiIgnore),

            // csi intermediate -> ground w/ csi dispatch
            (State::CsiIntermediate, '\u{40}'..='\u{7e}') => {
                self.state = State::Ground;
                self.csi_dispatch(terminal, input);
            }

            // anywhere -> osc string
            (_, '\u{9d}') => self.enter(State::OscString),

            // osc string event / ignore
            (State::OscString, '\u{00}'..='\u{17}') => (),
            (State::OscString, '\u{19}') => (),
            (State::OscString, '\u{1c}'..='\u{1f}') => (),

            // osc string event / osc put (unhandled)
            (State::OscString, '\u{20}'..='\u{7f}') => (),

            // anywhere -> dcs entry
            (_, '\u{90}') => self.enter(State::DcsEntry),

            // dcs entry event / ignore
            (State::DcsEntry, '\u{00}'..='\u{17}') => (),
            (State::DcsEntry, '\u{19}') => (),
            (State::DcsEntry, '\u{1c}'..='\u{1f}') => (),
            (State::DcsEntry, '\u{7f}') => (),

            // dcs entry -> dcs param w/ param
            (State::DcsEntry, '\u{30}'..='\u{39}') | (State::DcsEntry, '\u{3b}') => {
                self.enter(State::DcsParam);
                self.param(input);
            }

            // dcs entry -> dcs param w/ collect
            (State::DcsEntry, '\u{3c}'..='\u{3f}') => {
                self.enter(State::DcsParam);
                self.collect(input);
            }

            // dcs entry -> dcs ignore
            (State::DcsEntry, '\u{3a}') => self.enter(State::DcsIgnore),

            // dcs entry -> dcs intermediate w/ collect
            (State::DcsEntry, '\u{20}'..='\u{2f}') => {
                self.enter(State::DcsIntermediate);
                self.collect(input);
            }

            // dcs entry -> dcs passthrough
            (State::DcsEntry, '\u{40}'..='\u{7e}') => self.enter(State::DcsPassthrough),

            // dcs param event / ignore
            (State::DcsParam, '\u{00}'..='\u{17}') => (),
            (State::DcsParam, '\u{19}') => (),
            (State::DcsParam, '\u{1c}'..='\u{1f}') => (),

            // dcs param event / param
            (State::DcsParam, '\u{30}'..='\u{39}') => self.param(input),
            (State::DcsParam, '\u{3b}') => self.param(input),

            // dcs param event / ignore
            (State::DcsParam, '\u{7f}') => (),

            // dcs param -> dcs ignore
            (State::DcsParam, '\u{3a}') => self.enter(State::DcsIgnore),
            (State::DcsParam, '\u{3c}'..='\u{3f}') => self.enter(State::DcsIgnore),

            // dcs param -> dcs intermediate w/ collect
            (State::DcsParam, '\u{20}'..='\u{2f}') => {
                self.enter(State::DcsIntermediate);
                self.collect(input);
            }

            // dcs param -> dcs passthrough
            (State::DcsParam, '\u{40}'..='\u{7e}') => self.enter(State::DcsPassthrough),

            // dcs ignore event / ignore
            (State::DcsIgnore, '\u{00}'..='\u{17}') => (),
            (State::DcsIgnore, '\u{19}') => (),
            (State::DcsIgnore, '\u{1c}'..='\u{1f}') => (),
            (State::DcsIgnore, '\u{20}'..='\u{7f}') => (),

            // dcs intermediate event / ignore
            (State::DcsIntermediate, '\u{00}'..='\u{17}') => (),
            (State::DcsIntermediate, '\u{19}') => (),
            (State::DcsIntermediate, '\u{1c}'..='\u{1f}') => (),

            // dcs intermediate event / collect
            (State::DcsIntermediate, '\u{20}'..='\u{2f}') => self.collect(input),

            // dcs intermediate event / ignore
            (State::DcsIntermediate, '\u{7f}') => (),

            // dcs intermediate event -> dcs ignore
            (State::DcsIntermediate, '\u{30}'..='\u{3f}') => self.enter(State::DcsIgnore),

            // dcs intermediate event -> dcs passthrough
            (State::DcsIntermediate, '\u{40}'..='\u{7e}') => self.enter(State::DcsPassthrough),

            // dcs passthrough event / put (unhandled)
            (State::DcsPassthrough, '\u{00}'..='\u{17}') => (),
            (State::DcsPassthrough, '\u{19}') => (),
            (State::DcsPassthrough, '\u{1c}'..='\u{1f}') => (),
            (State::DcsPassthrough, '\u{20}'..='\u{7e}') => (),

            // dcs passthrough event / ignore
            (State::DcsPassthrough, '\u{7f}') => (),

            // anywhere -> sos pm apc string
            (_, '\u{98}') | (_, '\u{9e}') | (_, '\u{9f}') => self.enter(State::SosPmApcString),

            // sos pm apc string event / ignore
            (State::SosPmApcString, '\u{00}'..='\u{17}') => (),
            (State::SosPmApcString, '\u{19}') => (),
            (State::SosPmApcString, '\u{1c}'..='\u{1f}') => (),
            (State::SosPmApcString, '\u{20}'..='\u{7f}') => (),

            _ => panic!(
                "Unhandled state / input pair: {:?} / {:x} ",
                self.state, input as u32
            ),
        }
    }

    fn enter(&mut self, state: State) {
        match state {
            // No constant events, different entry points may trigger own
            State::Ground => (),
            State::EscapeIntermediate => (),
            State::SosPmApcString => (),
            State::CsiParam => (),
            State::CsiIgnore => (),
            State::CsiIntermediate => (),
            State::DcsParam => (),
            State::DcsIgnore => (),
            State::DcsIntermediate => (),
            // Alway run clear
            State::Escape => self.clear(),
            State::CsiEntry => self.clear(),
            State::DcsEntry => self.clear(),
            // Should be osc start, currently unhandled
            State::OscString => (),
            // Should run hook, currently unhandled
            State::DcsPassthrough => (),
        }
        self.state = state;
    }

    fn execute(&mut self, terminal: &mut Terminal, input: char) {
        match input {
            '\u{08}' => terminal.bs(),
            '\u{09}' => terminal.ht(),
            '\u{0a}' => terminal.lf(),
            '\u{0b}' => terminal.lf(),
            '\u{0c}' => terminal.lf(),
            '\u{0d}' => terminal.cr(),
            '\u{0e}' => terminal.so(),
            '\u{0f}' => terminal.si(),
            '\u{84}' => terminal.lf(),
            '\u{85}' => terminal.nel(),
            '\u{88}' => terminal.hts(),
            '\u{8d}' => terminal.ri(),
            _ => (),
        }
    }

    fn clear(&mut self) {
        self.params = Params::default();
        self.intermediates = Intermediates::default();
    }

    fn collect(&mut self, input: char) {
        self.intermediates.0.push(input);
    }

    fn param(&mut self, input: char) {
        if input == ';' {
            self.params.0.push(0);
        } else {
            let n = self.params.0.len() - 1;
            let p = &mut self.params.0[n];
            *p = (10 * (*p as u32) + (input as u32) - 0x30) as u16;
        }
    }

    fn esc_dispatch(&mut self, terminal: &mut Terminal, input: char) {
        match (self.intermediates.0.first(), input) {
            (None, c) if ('@'..='_').contains(&c) => {
                self.execute(terminal, ((input as u8) + 0x40) as char)
            }
            (None, '7') => terminal.sc(),
            (None, '8') => terminal.rc(),
            (None, 'c') => {
                self.state = State::Ground;
                terminal.ris();
            }
            (Some('#'), '8') => terminal.decaln(),
            (Some('('), '0') => terminal.gzd4(Charset::Drawing),
            (Some('('), _) => terminal.gzd4(Charset::Ascii),
            (Some(')'), '0') => terminal.g1d4(Charset::Drawing),
            (Some(')'), _) => terminal.g1d4(Charset::Ascii),
            _ => (),
        }
    }

    fn csi_dispatch(&mut self, terminal: &mut Terminal, input: char) {
        match (self.intermediates.0.first(), input) {
            (None, '@') => terminal.ich(&self.params),
            (None, 'A') => terminal.cuu(&self.params),
            (None, 'B') => terminal.cud(&self.params),
            (None, 'C') => terminal.cuf(&self.params),
            (None, 'D') => terminal.cub(&self.params),
            (None, 'E') => terminal.cnl(&self.params),
            (None, 'F') => terminal.cpl(&self.params),
            (None, 'G') => terminal.cha(&self.params),
            (None, 'H') => terminal.cup(&self.params),
            (None, 'I') => terminal.cht(&self.params),
            (None, 'J') => terminal.ed(&self.params),
            (None, 'K') => terminal.el(&self.params),
            (None, 'L') => terminal.il(&self.params),
            (None, 'M') => terminal.dl(&self.params),
            (None, 'P') => terminal.dch(&self.params),
            (None, 'S') => terminal.su(&self.params),
            (None, 'T') => terminal.sd(&self.params),
            (None, 'W') => terminal.ctc(&self.params),
            (None, 'X') => terminal.ech(&self.params),
            (None, 'Z') => terminal.cbt(&self.params),
            (None, '`') => terminal.cha(&self.params),
            (None, 'a') => terminal.cuf(&self.params),
            (None, 'b') => terminal.rep(&self.params),
            (None, 'd') => terminal.vpa(&self.params),
            (None, 'e') => terminal.vpr(&self.params),
            (None, 'f') => terminal.cup(&self.params),
            (None, 'g') => terminal.tbc(&self.params),
            (None, 'h') => terminal.sm(&self.params),
            (None, 'l') => terminal.rm(&self.params),
            (None, 'm') => terminal.sgr(&self.params),
            (None, 'r') => terminal.decstbm(&self.params),
            (None, 's') => terminal.sc(),
            (None, 't') => terminal.xtwinops(&self.params),
            (None, 'u') => terminal.rc(),
            (Some('!'), 'p') => terminal.decstr(),
            (Some('?'), 'h') => terminal.prv_sm(&self.params),
            (Some('?'), 'l') => terminal.prv_rm(&self.params),
            _ => {}
        }
    }
}
