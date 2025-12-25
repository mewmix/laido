use crate::combat::judge_outcome;
use crate::config::*;
use crate::rng::XorShift32;
use crate::types::*;

#[derive(Clone, Debug)]
pub struct DuelConfig {
    pub seed: u32,
    pub clash: bool,
}

impl Default for DuelConfig {
    fn default() -> Self { Self { seed: 0xA1D0_5EED, clash: true } }
}

#[derive(Clone, Debug)]
pub struct DuelMachine {
    pub phase: DuelPhase,
    pub rng: XorShift32,
    pub seed: u32,
    pub human_opening: Opening,
    pub ai_opening: Opening,
    pub go_ts_ms: Option<u64>,
    pub phase_start_ms: u64,
    pub delay_target_ms: Option<u64>,
    pub human_swipe: Option<SwipeEvent>,
    pub ai_swipe: Option<SwipeEvent>,
    pub round_results: Vec<RoundResult>,
    pub round_meta: Vec<RoundMeta>,
    pub match_state: MatchState,
    pub human_score: u8,
    pub ai_score: u8,
    pub input_window_ms: u64,
}

impl DuelMachine {
    pub fn new(cfg: DuelConfig, start_ms: u64) -> Self {
        let mut rng = XorShift32::new(cfg.seed);
        let human_opening = pick_opening(&mut rng);
        let ai_opening = pick_opening(&mut rng);
        Self {
            phase: DuelPhase::Standoff,
            rng,
            seed: cfg.seed,
            human_opening,
            ai_opening,
            go_ts_ms: None,
            phase_start_ms: start_ms,
            delay_target_ms: None,
            human_swipe: None,
            ai_swipe: None,
            round_results: Vec::with_capacity(3),
            round_meta: Vec::with_capacity(3),
            match_state: MatchState::InProgress,
            human_score: 0,
            ai_score: 0,
            input_window_ms: INPUT_WINDOW_MS,
        }
    }

    pub fn current_opening(&self) -> Opening { self.human_opening }

    pub fn schedule_go_delay(&mut self) -> u64 {
        self.rng.range_u64(RANDOM_DELAY_MIN_MS, RANDOM_DELAY_MAX_MS)
    }

    pub fn schedule_clash_delay(&mut self) -> u64 {
        self.rng.range_u64(CLASH_DELAY_MIN_MS, CLASH_DELAY_MAX_MS)
    }

    pub fn start_round(&mut self, now_ms: u64) { self.enter_random_delay(now_ms, false); }

    fn enter_random_delay(&mut self, now_ms: u64, clash: bool) {
        self.phase = DuelPhase::RandomDelay;
        self.phase_start_ms = now_ms;
        self.go_ts_ms = None;
        self.human_swipe = None;
        self.ai_swipe = None;
        self.input_window_ms = if clash { CLASH_INPUT_WINDOW_MS } else { INPUT_WINDOW_MS };
        self.human_opening = pick_opening(&mut self.rng);
        self.ai_opening = pick_opening(&mut self.rng);
        let delay = if clash { self.schedule_clash_delay() } else { self.schedule_go_delay() };
        self.delay_target_ms = Some(now_ms + delay);
    }

    pub fn tick(&mut self, now_ms: u64) {
        match self.phase {
            DuelPhase::Standoff => {
                if now_ms - self.phase_start_ms >= START_DELAY_MS {
                    self.start_round(now_ms);
                }
            }
            DuelPhase::RandomDelay => {
                if let Some(target) = self.delay_target_ms {
                    if now_ms >= target {
                        self.phase = DuelPhase::GoSignal;
                        self.go_ts_ms = Some(now_ms);
                        self.phase_start_ms = now_ms;
                        self.delay_target_ms = None;
                    }
                }
            }
            DuelPhase::GoSignal => {
                self.phase = DuelPhase::InputWindow;
                self.phase_start_ms = now_ms;
            }
            DuelPhase::InputWindow => {
                if now_ms - self.phase_start_ms >= self.input_window_ms {
                    // Resolve immediately at window end to avoid extra frame dependency
                    let outcome = self.resolve(now_ms);
                    self.apply_outcome(outcome);
                    self.phase = DuelPhase::ResultFlash;
                    self.phase_start_ms = now_ms;
                }
            }
            DuelPhase::Resolution => {
                let outcome = self.resolve(now_ms);
                self.apply_outcome(outcome);
                self.phase = DuelPhase::ResultFlash;
                self.phase_start_ms = now_ms;
            }
            DuelPhase::ResultFlash => {
                if now_ms - self.phase_start_ms >= 300 { // ≤300 ms flash
                    self.phase = DuelPhase::NextRound;
                    self.phase_start_ms = now_ms;
                }
            }
            DuelPhase::NextRound => {
                if self.match_state != MatchState::InProgress {
                    self.phase = DuelPhase::Finished;
                } else if now_ms - self.phase_start_ms >= 500 { // ≤500 ms reset
                    self.enter_random_delay(now_ms, false);
                }
            }
            DuelPhase::Finished | DuelPhase::Reset => {}
        }
    }

    pub fn on_swipe(&mut self, actor: Actor, dir: Direction, ts_ms: u64) {
        // Any swipe before GO is instant loss for that actor
        if let Some(go) = self.go_ts_ms {
            if ts_ms < go {
                let outcome = match actor { Actor::Human => Outcome::EarlyHuman, Actor::Ai => Outcome::EarlyAi };
                self.round_results.push(RoundResult { 
                    human_opening: self.human_opening, 
                    ai_opening: self.ai_opening, 
                    outcome, 
                    human_reaction_ms: None, 
                    ai_reaction_ms: None 
                });
                self.phase = DuelPhase::ResultFlash;
                self.phase_start_ms = ts_ms;
                match actor { Actor::Human => self.ai_score += 1, Actor::Ai => self.human_score += 1 }
                ;
                self.update_match_state();
                return;
            }
        } else {
            // GO not scheduled yet => early
            let outcome = match actor { Actor::Human => Outcome::EarlyHuman, Actor::Ai => Outcome::EarlyAi };
            self.round_results.push(RoundResult { 
                human_opening: self.human_opening, 
                ai_opening: self.ai_opening, 
                outcome, 
                human_reaction_ms: None, 
                ai_reaction_ms: None 
            });
            self.phase = DuelPhase::ResultFlash;
            self.phase_start_ms = ts_ms;
            match actor { Actor::Human => self.ai_score += 1, Actor::Ai => self.human_score += 1 }
            ;
            self.update_match_state();
            return;
        }

        if self.phase != DuelPhase::InputWindow { return; }
        let go = self.go_ts_ms.unwrap();
        if ts_ms - go > self.input_window_ms { return; }
        let ev = SwipeEvent { dir, ts_ms };
        match actor {
            Actor::Human => if self.human_swipe.is_none() { self.human_swipe = Some(ev); },
            Actor::Ai => if self.ai_swipe.is_none() { self.ai_swipe = Some(ev); },
        }
    }

    fn resolve(&mut self, _now_ms: u64) -> Outcome {
        let go = self.go_ts_ms.unwrap_or(self.phase_start_ms);
        let human_dir = self.human_swipe.as_ref().map(|e| e.dir);
        let ai_dir = self.ai_swipe.as_ref().map(|e| e.dir);
        let human_r = self.human_swipe.as_ref().map(|e| e.ts_ms - go);
        let ai_r = self.ai_swipe.as_ref().map(|e| e.ts_ms - go);
        let outcome = judge_outcome(
            self.human_opening,
            self.ai_opening,
            human_dir,
            ai_dir,
            human_r,
            ai_r,
            TIE_WINDOW_MS,
        );
        // Store metadata and result (preallocated capacity prevents allocs during duel)
        self.round_meta.push(RoundMeta { go_ts_ms: go, human: self.human_swipe.clone(), ai: self.ai_swipe.clone() });
        self.round_results.push(RoundResult {
            human_opening: self.human_opening,
            ai_opening: self.ai_opening,
            outcome,
            human_reaction_ms: human_r.map(|v| v as u32),
            ai_reaction_ms: ai_r.map(|v| v as u32),
        });
        outcome
    }

    fn apply_outcome(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::HumanWin | Outcome::WrongAi | Outcome::EarlyAi => self.human_score += 1,
            Outcome::AiWin | Outcome::WrongHuman | Outcome::EarlyHuman => self.ai_score += 1,
            Outcome::Clash => {
                // Immediate rematch with reduced delay/window
                let now_ms = self.phase_start_ms;
                self.enter_random_delay(now_ms, true);
                return;
            }
        }
        self.update_match_state();
    }

    fn update_match_state(&mut self) {
        if self.human_score >= ROUNDS_TO_WIN { self.match_state = MatchState::HumanWon; }
        else if self.ai_score >= ROUNDS_TO_WIN { self.match_state = MatchState::AiWon; }
        else { self.match_state = MatchState::InProgress; }
    }

    // Export last round as a DuelLog for deterministic replay
    pub fn last_duel_log(&self) -> Option<crate::logging::DuelLog> {
        let i = self.round_results.len().checked_sub(1)?;
        let rr = &self.round_results[i];
        let meta = &self.round_meta[i];
        Some(crate::logging::DuelLog {
            seed: self.seed,
            human_opening: rr.human_opening,
            ai_opening: rr.ai_opening,
            go: GoEvent { ts_ms: meta.go_ts_ms },
            human: meta.human.clone(),
            ai: meta.ai.clone(),
            outcome: rr.outcome,
        })
    }

    #[cfg(test)]
    pub fn force_go(&mut self, now_ms: u64) { self.phase = DuelPhase::GoSignal; self.go_ts_ms = Some(now_ms); self.phase_start_ms = now_ms; }

    pub fn open_input(&mut self, now_ms: u64) { self.phase = DuelPhase::InputWindow; self.go_ts_ms = Some(now_ms); self.phase_start_ms = now_ms; }

    pub fn reset_match(&mut self, now_ms: u64) {
        self.phase = DuelPhase::Standoff;
        self.go_ts_ms = None;
        self.phase_start_ms = now_ms;
        self.delay_target_ms = None;
        self.human_swipe = None;
        self.ai_swipe = None;
        self.round_results.clear();
        self.round_meta.clear();
        self.match_state = MatchState::InProgress;
        self.human_score = 0;
        self.ai_score = 0;
        self.input_window_ms = INPUT_WINDOW_MS;
        self.human_opening = pick_opening(&mut self.rng);
        self.ai_opening = pick_opening(&mut self.rng);
    }
}

pub fn pick_opening(rng: &mut XorShift32) -> Opening {
    match rng.next_u32() % 10 {
        0 => Opening::Up,
        1 => Opening::UpRight,
        2 => Opening::Right,
        3 => Opening::DownRight,
        4 => Opening::Down,
        5 => Opening::DownLeft,
        6 => Opening::Left,
        7 => Opening::UpLeft,
        8 => Opening::UpDown,
        _ => Opening::LeftRight,
    }
}